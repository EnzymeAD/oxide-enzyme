// TODO: Verify wether to import the LLVM* from enzyme_sys, or from llvm-sys
use enzyme_sys::{
    CDerivativeMode, CFnTypeInfo, EnzymeCreatePrimalAndGradient, EnzymeLogicRef,
    EnzymeTypeAnalysisRef, FreeEnzymeLogic, FreeTypeAnalysis, IntList,
};
use enzyme_sys::{CreateEnzymeLogic, CreateTypeAnalysis, EnzymeSetCLBool, LLVMValueRef};
pub use enzyme_sys::{LLVMOpaqueValue, CDIFFE_TYPE};

use super::enzyme_sys;
use super::tree::TypeTree;

use std::os::raw::c_void;
use std::ptr;

pub fn enzyme_print_activity(val: bool) {
    #[link(name = "Enzyme-13")]
    extern "C" {
        static mut EnzymePrintActivity: c_void;
    }
    unsafe {
        EnzymeSetCLBool(std::ptr::addr_of_mut!(EnzymePrintActivity), val as u8);
    }
}

pub fn enzyme_print_type(val: bool) {
    #[link(name = "Enzyme-13")]
    extern "C" {
        static mut EnzymePrintType: c_void;
    }
    unsafe {
        EnzymeSetCLBool(std::ptr::addr_of_mut!(EnzymePrintType), val as u8);
    }
}

pub fn enzyme_print_functions(val: bool) {
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
    pub ret_info: ReturnActivity,
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
    pub fn new(
        primary_name: &str,
        grad_name: &str,
        input_activity: Vec<CDIFFE_TYPE>,
        ret_info: ReturnActivity,
    ) -> FncInfo {
        FncInfo {
            primary_name: primary_name.to_string(),
            grad_name: grad_name.to_string(),
            params: ParamInfos {
                input_activity,
                ret_info,
            },
        }
    }
}

// The Enzyme API is too unspecific for the return type, so we introduced
// the stricter CDIFFE_RETTYPE to not allow types which are illegal for
// the ret activity. Enzyme doesn't know this type, so we match it back.
// We should add the capability to enzyme to also support DUP_ARG on merged
// forward+reverse however.
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum ReturnActivity {
    Active,
    Gradient,
    Constant,
    Ignore,
    None,
}
#[repr(u32)]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FwdReturnActivity {
    Active,
    Gradient,
}

pub struct AutoDiff {
    logic_ref: EnzymeLogicRef,
    type_analysis: EnzymeTypeAnalysisRef,
}

impl AutoDiff {
    pub fn new(opt: bool) -> Self {
        let logic_ref = unsafe { CreateEnzymeLogic(opt as u8) };
        let type_analysis =
            unsafe { CreateTypeAnalysis(logic_ref, ptr::null_mut(), ptr::null_mut(), 0) };
        AutoDiff {
            logic_ref,
            type_analysis,
        }
    }

    pub fn create_fwd_diff(
        &self,
        fnc_todiff: LLVMValueRef,
        args_activity: &mut [CDIFFE_TYPE],
        ret_info: FwdReturnActivity,
    ) {
    }

    pub fn create_primal_and_gradient(
        &self,
        fnc_todiff: LLVMValueRef,
        args_activity: &mut [CDIFFE_TYPE],
        ret_info: ReturnActivity,
    ) -> LLVMValueRef {
        let (ret_activity, ret_primary_ret) = match ret_info {
            ReturnActivity::Active => (CDIFFE_TYPE::DFT_OUT_DIFF, true as u8),
            ReturnActivity::Gradient => (CDIFFE_TYPE::DFT_OUT_DIFF, false as u8),
            ReturnActivity::Constant => (CDIFFE_TYPE::DFT_CONSTANT, true as u8),
            ReturnActivity::Ignore => (CDIFFE_TYPE::DFT_CONSTANT, false as u8),
            ReturnActivity::None => (CDIFFE_TYPE::DFT_CONSTANT, false as u8), // those should be ignored by enzyme since we don't have a return, just a safe fallback
        };

        let tree_tmp = TypeTree::new();

        let mut args_tree = vec![tree_tmp.inner; args_activity.len()];

        // We don't support volatile / extern / (global?) values.
        // Just because I didn't had time to test them, and it seems less urgent.
        let mut args_uncacheable = vec![0; args_activity.len()];

        //let ret = tree::TypeTree::from_type(CConcreteType::DT_Float, context).prepend(0);
        let ret = TypeTree::new();

        let kv_tmp = IntList {
            data: ptr::null_mut(),
            size: 0,
        };

        let mut known_values = vec![kv_tmp; args_activity.len()];

        let dummy_type = CFnTypeInfo {
            Arguments: args_tree.as_mut_ptr(),
            Return: ret.inner,
            KnownValues: known_values.as_mut_ptr(),
        };

        dbg!("before-ad");
        let res = unsafe {
            EnzymeCreatePrimalAndGradient(
                self.logic_ref, // Logic
                fnc_todiff,
                ret_activity, // LLVM function, return type
                args_activity.as_mut_ptr(),
                args_activity.len() as u64, // constant arguments
                self.type_analysis,         // type analysis struct
                ret_primary_ret as u8,
                0,                                        //0
                CDerivativeMode::DEM_ReverseModeCombined, // return value, dret_used, top_level which was 1
                1,                                        // vector mode width
                1,                                        // free memory
                ptr::null_mut(),
                dummy_type, // additional_arg, type info (return + args)
                args_uncacheable.as_mut_ptr(),
                args_uncacheable.len() as u64, // uncacheable arguments
                ptr::null_mut(),               // write augmented function to this
                0,
            )
        };
        dbg!("after-ad");
        res
    }
}

impl Drop for AutoDiff {
    fn drop(&mut self) {
        unsafe {
            FreeTypeAnalysis(self.type_analysis);
            FreeEnzymeLogic(self.logic_ref);
        }
    }
}
