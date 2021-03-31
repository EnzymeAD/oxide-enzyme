#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/enzyme.rs"));

pub mod tree;
pub mod typeinfo;

use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::prelude::LLVMModuleRef; // added by myself

/*
pub struct AutoDiff {
    aa_results_ref: EnzymeAAResultsRef,
    type_analysis: EnzymeTypeAnalysisRef
}

impl AutoDiff {
    pub fn new(module: LLVMModuleRef, type_analysis: EnzymeTypeAnalysisRef) -> AutoDiff {
        let aa_results_ref = unsafe { EnzymeGetGlobalAA(module) };

        AutoDiff { aa_results_ref, type_analysis }
    }

    pub fn create_primal_and_gradient(&self, fnc: LLVMValueRef, retType: CDIFFE_TYPE, args: Vec<CDIFFE_TYPE>, type_info: typeinfo::TypeInfo) {

    }
}

impl Drop for AutoDiff {
    fn drop(&mut self) {
        unsafe { EnzymeFreeGlobalAA(self.aa_results_ref) }
    }
}*/

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
    fn build_tree() {

    }

    #[test]
    fn get_global_aa() {
        let dummy_module = unsafe {
            LLVMModuleCreateWithName(CString::new("dummy").unwrap().into_raw())
        } as *mut LLVMOpaqueModule;
    }
}

