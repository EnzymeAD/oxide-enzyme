#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/enzyme.rs"));

// TODO check where we should change the generated bindings and remove the mut. Apparently it's added everywhere (?), but enzyme handles quite a few args as const.


pub mod tree;
pub mod typeinfo;

use std::ffi::CString;
//use llvm_sys::prelude::LLVMValueRef;

pub fn createEmptyTypeAnalysis() -> EnzymeTypeAnalysisRef {
    let tripple = CString::new("x86_64-unknown-linux-gnu").unwrap().into_raw();
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

    /*
    pub fn create_primal_and_gradient(&self, fnc_todiff: LLVMValueRef, retType: CDIFFE_TYPE, args: Vec<CDIFFE_TYPE>, type_info: typeinfo::TypeInfo) {
      let foo: LLVMValueRef = unsafe {
        EnzymeCreatePrimalAndGradient(self.logic_ref, fnc_todiff, retType, args.as_mut_ptr(), args.len(), self.type_analysis, 
         returnValue: u8,
         dretUsed: u8,
         topLevel: u8, ??
         additionalArg: LLVMTypeRef,
         typeInfo: CFnTypeInfo,
         _uncacheable_args: *mut u8, // simple test => no uncacheable args
         uncacheable_args_size: size_t, // 0
         augmented: EnzymeAugmentedReturnPtr,
         AtomicAdd: u8, //doesn't matter, lets say true
         PostOpt: u8, // sound's good
      }
    }*/
}

impl Drop for AutoDiff {
    fn drop(&mut self) {
        unsafe { FreeEnzymeLogic(self.logic_ref) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llvm_sys::core::{LLVMContextCreate, LLVMModuleCreateWithName};
    use std::ffi::CString;

    #[test]
    fn empty_tree() {
        let _ = unsafe {
            EnzymeNewTypeTree()
        };
    }


    #[test]
    fn new_type_analysis() {
      let _ta = createEmptyTypeAnalysis();
    }

    #[test]
    fn new_autodiff() {
      let ta = createEmptyTypeAnalysis();
      let _ad = AutoDiff::new(ta);
    }

    #[test]
    fn get_LLVM_Module() {
        let _dummy_module = unsafe {
            LLVMModuleCreateWithName(CString::new("dummy").unwrap().into_raw())
        } as *mut LLVMOpaqueModule;
    }
    #[test]
    fn basic_autodiff() {
      2;
    }

    fn square(x: f32) -> f32 {
      x * x
    }
  
    /*
    #[test]
    fn dsquare() {
      let epsilon = 1e-3;
      let v1 = __enzyme_autodiff(square, 1.);
      let v2 = __enzyme_autodiff(square, 2.);
      let v3 = __enzyme_autodiff(square, 2.5);
      assert!(v1- 2. < epsilon);
      assert!(v1- 4. < epsilon);
      assert!(v1- 5. < epsilon);
    }*/


}
