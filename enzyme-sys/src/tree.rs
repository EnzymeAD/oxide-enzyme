use std::ffi::{CStr, CString};
use std::fmt;

use crate::{CTypeTreeRef, EnzymeFreeTypeTree, EnzymeNewTypeTree, EnzymeNewTypeTreeCT, CConcreteType, EnzymeTypeTreeOnlyEq, EnzymeMergeTypeTree, EnzymeTypeTreeShiftIndiciesEq, EnzymeTypeTreeToString, EnzymeTypeTreeToStringFree};
use crate::LLVMOpaqueContext;

pub struct TypeTree {
    pub inner: CTypeTreeRef
}

impl TypeTree {
    pub fn new() -> TypeTree {
        let inner = unsafe { EnzymeNewTypeTree() };

        TypeTree { inner }
    }

    pub fn from_type(t: CConcreteType, ctx: *mut LLVMOpaqueContext) -> TypeTree {
        let inner = unsafe { EnzymeNewTypeTreeCT(t, ctx) };

        TypeTree { inner }
    }

    pub fn prepend(self, idx: isize) -> Self {
        unsafe { 
            EnzymeTypeTreeOnlyEq(self.inner, idx as i64)
        }

        self
    }

    pub fn merge_with(self, other: Self) -> Self {
        unsafe {
            EnzymeMergeTypeTree(self.inner, other.inner);
        }

        drop(other);
        self
    }

    pub fn shift_indices(self, layout: &str, offset: isize, max_size: isize, add_offset: usize) -> Self {
        let layout = CString::new(layout).unwrap();

        unsafe {
            EnzymeTypeTreeShiftIndiciesEq(self.inner, layout.as_ptr(), offset as i64, max_size as i64, add_offset as u64)
        }

        self
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

