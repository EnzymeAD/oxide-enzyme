use std::ffi::CStr;
use std::fmt;

use super::enzyme_sys::{CTypeTreeRef, EnzymeFreeTypeTree, EnzymeNewTypeTree, EnzymeTypeTreeToString, EnzymeTypeTreeToStringFree};

pub struct TypeTree {
    pub inner: CTypeTreeRef
}

// Covered:
// CTypeTreeRef
// EnzymeNewTypeTree();
// EnzymeNewTypeTreeCT
// EnzymeMergeTypeTree
// EnzymeFreeTypeTree
// EnzymeTypeTreeOnlyEq
// EnzymeTypeTreeShiftIndiciesEq
// EnzymeTypeTreeToString
// EnzymeTypeTreeToStringFree

// NOT Covered, TODO:
// EnzymeCreatePrimalAndGradient
// EnzymeCreateAugmentedPrimal
//
// TA part:
// CreateTypeAnalysis
// ClearTypeAnalysis
// FreeTypeAnalysis
// 
// Logic part:
// CreateEnzymeLogic
// ClearEnzymeLogic
// FreeEnzymeLogic
// 


// NOT Covered, relevant?:
// EnzymeNewTypeTreeTR
// EnzymeSetTypeTree
// EnzymeTypeTreeData0Eq
// EnzymeSetCLBool
// EnzymeSetCLInteger
// EnzymeExtractReturnInfo
// EnzymeExtractFunctionFromAugmentation
// EnzymeExtractTapeTypeFromAugmentation
// EnzymeRegisterAllocationHandler <= prob. not relevant for now

impl TypeTree {
    pub fn new() -> TypeTree {
        let inner = unsafe { EnzymeNewTypeTree() };

        TypeTree { inner }
    }
}

impl fmt::Display for TypeTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = unsafe { 
                EnzymeTypeTreeToString(self.inner)
        };
        let cstr = unsafe {
            CStr::from_ptr(ptr)
        };
        match cstr.to_str() {
            Ok(x) => write!(f, "{}", x)?,
            Err(err) => write!(f, "could not parse: {}", err)?
        }

        // delete C string pointer
        unsafe {
            EnzymeTypeTreeToStringFree(ptr)
        }

        Ok(())
    }
}

impl Drop for TypeTree {
    fn drop(&mut self) {
        unsafe {
            EnzymeFreeTypeTree(self.inner)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llvm_sys::core::LLVMContextCreate;

    fn create_context() -> *mut LLVMOpaqueContext {
        let tmp = unsafe { LLVMContextCreate() };

        tmp as *mut LLVMOpaqueContext
    }

    #[test]
    fn create_tree() {
        let context = create_context();

        let n1 = TypeTree::from_type(CConcreteType::DT_Pointer, context)
            .prepend(1)
            .prepend(0);

        assert_eq!(n1.to_string(), "{[0,1]:Pointer}");

        let n2 = TypeTree::from_type(CConcreteType::DT_Float, context)
            .prepend(2);

        assert_eq!(n2.to_string(), "{[2]:Float@float}");

        let n3 = n1.merge_with(n2)
            .prepend(4);

        assert_eq!(n3.to_string(), "{[4,0,1]:Pointer, [4,2]:Float@float}");
    }

    /*
    #[test]
    #[should_panic]
    fn not_unique_repeating() {
        let context = create_context();

        let n1 = TypeTree::from_type(CConcreteType::DT_Float, context);
        let n2 = TypeTree::from_type(CConcreteType::DT_Float, context)
            .prepend(-1);

        let n3 = n1.merge_with(n2)
            .prepend(0);

        dbg!(&n3.inner);
    }*/
}

//dwarf metadata in llvm-ir und konvertieren in type_tree: https://docs.rs/llvm-sys/110.0.1/llvm_sys/debuginfo/index.html
