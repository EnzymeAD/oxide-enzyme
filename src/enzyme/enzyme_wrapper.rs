// TODO: Verify wether to import the LLVM* from enzyme_sys, or from llvm-sys
use enzyme_sys::{LLVMValueRef, CreateTypeAnalysis, CreateEnzymeLogic, EnzymeSetCLBool};
use enzyme_sys::{EnzymeLogicRef, FreeEnzymeLogic, EnzymeCreatePrimalAndGradient, CFnTypeInfo, IntList, EnzymeTypeAnalysisRef, CDerivativeMode};
pub use enzyme_sys::{LLVMOpaqueContext, LLVMOpaqueValue, CDIFFE_TYPE};

use super::enzyme_sys;
use super::tree::TypeTree;

use std::ffi::CString;
use std::ptr;
use std::os::raw::c_void;

pub fn enzyme_set_clbool(val: bool) {
    #[link(name = "Enzyme-13")]
    extern "C" {
        static mut EnzymePrint: c_void;
    }
    unsafe {
        EnzymeSetCLBool(std::ptr::addr_of_mut!(EnzymePrint), val as u8);
    }
}

/// Should be given by Enzyme users to declare how arguments shall be handled
#[derive(Clone)]
pub struct FncInfo {
    pub primary_name: String, // What's the (unmangled) name of the Rust function to differentiate?
    pub grad_name: String,
    pub params: ParamInfos,
}

#[derive(Clone)]
pub struct ParamInfos {
    pub input_activity: Vec<CDIFFE_TYPE>, // How should it's arguments be treated?
    pub ret_info: Option<(CDIFFE_TYPE, bool)>,
}

impl FncInfo {
    /// Enzyme requires one FncInfo Struct per function differentiation
    ///
    /// primary_name should be identical to the name of the existing rust function.
    ///
    /// grad_name will be the name of the generated rust function.
    ///
    /// ret_info should be None if the primary function has no return type.
    /// Otherwise it should specify if we want the output's gradient and if we want
    /// the return value of the primal rust function.
    pub fn new(primary_name: &str, grad_name: &str, input_activity: Vec<CDIFFE_TYPE>, ret_info: Option<(CDIFFE_TYPE, bool)>) -> FncInfo {
        FncInfo { 
            primary_name: primary_name.to_string(), 
            grad_name: grad_name.to_string(), 
            params: ParamInfos {input_activity, ret_info},
        }
    }
}


pub fn create_empty_type_analysis() -> EnzymeTypeAnalysisRef {
    let platform: String = std::env::var("TARGET").unwrap();
    let tripple = CString::new(platform).unwrap().into_raw();
    unsafe {
      CreateTypeAnalysis(tripple, std::ptr::null_mut(), std::ptr::null_mut(), 0)
    }
}

pub struct AutoDiff {
    logic_ref: EnzymeLogicRef,
    type_analysis: EnzymeTypeAnalysisRef
}

impl AutoDiff {
    pub fn new(type_analysis: EnzymeTypeAnalysisRef) -> AutoDiff {
        
        let logic_ref = unsafe { CreateEnzymeLogic() };
        AutoDiff { logic_ref, type_analysis }
    }

    pub fn create_primal_and_gradient(
        &self, fnc_todiff: LLVMValueRef, args_activity: &mut [CDIFFE_TYPE], 
        ret_info: Option<(CDIFFE_TYPE, bool)>, opt: bool) -> LLVMValueRef
    {

        let (ret_activity, ret_primary_ret) = match ret_info {
            None => (CDIFFE_TYPE::DFT_CONSTANT, false as u8),
            Some((activity, ret_primary_ret)) => (activity, ret_primary_ret as u8),
        };

        let tree_tmp = TypeTree::new();

        let mut args_tree = vec![tree_tmp.inner];

        // We don't support volatile / extern / (global?) values.
        // Just because I didn't had time to test them, and it seems less urgent.
        let mut args_uncacheable = vec![0;args_activity.len()];

        //let ret = tree::TypeTree::from_type(CConcreteType::DT_Float, context).prepend(0);
        let ret = TypeTree::new();

        let kv_tmp = IntList {
            data: ptr::null_mut(),
            size: 0,
        };

        let mut known_values = vec![kv_tmp];

        let dummy_type = CFnTypeInfo {
            Arguments: args_tree.as_mut_ptr(),
            Return: ret.inner,
            KnownValues: known_values.as_mut_ptr(),
        };

        unsafe {
            EnzymeCreatePrimalAndGradient(
                self.logic_ref, // Logic
                fnc_todiff, ret_activity, // LLVM function, return type
                args_activity.as_mut_ptr(), args_activity.len() as u64, // constant arguments
                self.type_analysis, // type analysis struct
                ret_primary_ret as u8, 0, CDerivativeMode::DEM_ReverseModeCombined, // return value, dret_used, top_level which was 1
                ptr::null_mut(), dummy_type, // additional_arg, type info (return + args)
                args_uncacheable.as_mut_ptr(), args_uncacheable.len() as u64, // uncacheable arguments
                ptr::null_mut(), // write augmented function to this
                0, opt as u8 // atomic_add, post_opt
            )
        }
    }
}

impl Drop for AutoDiff {
    fn drop(&mut self) {
        unsafe { FreeEnzymeLogic(self.logic_ref) }
    }
}
