#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/enzyme.rs"));

pub mod tree;

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

        unsafe {
            let tmp = EnzymeGetGlobalAA(dummy_module);
            EnzymeFreeGlobalAA(tmp);
        }
    }
}

