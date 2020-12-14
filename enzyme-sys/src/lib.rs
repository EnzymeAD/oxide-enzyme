#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/enzyme.rs"));

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
        // create LLVM context
        let context = unsafe {
            LLVMContextCreate()
        } as *mut LLVMOpaqueContext;

        // create two singleton tree within context
        let n1 = unsafe { EnzymeNewTypeTreeCT(CConcreteType::DT_Float, context) };
        let n2 = unsafe { EnzymeNewTypeTreeCT(CConcreteType::DT_Float, context) };

        assert_ne!(n1, n2);

        // combine them
        unsafe { EnzymeMergeTypeTree(n1, n2) };

        // get first item
        unsafe { EnzymeTypeTreeOnlyEq(n2, 4) };

        dbg!(&n1, &n2);

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

