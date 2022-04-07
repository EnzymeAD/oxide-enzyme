use super::enzyme_sys::{CTypeTreeRef, EnzymeFreeTypeTree, EnzymeNewTypeTree};

pub struct TypeTree {
    pub inner: CTypeTreeRef,
}

impl TypeTree {
    pub fn new() -> TypeTree {
        let inner = unsafe { EnzymeNewTypeTree() };

        TypeTree { inner }
    }
}

impl Drop for TypeTree {
    fn drop(&mut self) {
        unsafe { EnzymeFreeTypeTree(self.inner) }
    }
}

//dwarf metadata in llvm-ir und konvertieren in type_tree: https://docs.rs/llvm-sys/110.0.1/llvm_sys/debuginfo/index.html
