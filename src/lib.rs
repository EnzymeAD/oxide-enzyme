//extern crate llvm_sys;
//


use std::ptr;
use std::process;
use std::ffi::CString;

//use llvm_sys;
use llvm_sys::{core::*, debuginfo::LLVMGetSubprogram, execution_engine::LLVMCreateExecutionEngineForModule};
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction};
use llvm_sys::execution_engine::{LLVMGetFunctionAddress, LLVMLinkInMCJIT};
use llvm_sys::target::LLVM_InitializeNativeTarget;

pub fn pre_processing() {
    let out = process::Command::new("/usr/bin/rustc")
        .args(&["--emit=obj", "--emit=llvm-bc",  "src/main.rs", "-C", "debuginfo=2"])
        .output()
        .expect("failed to run cargo");

    dbg!(&out);
}

pub unsafe fn load_llvm() {
    let context = LLVMContextCreate();
    let mut msg = ptr::null_mut();

    let mut memory_buf = ptr::null_mut();
    let path = CString::new("./main.bc").unwrap();
    assert!(LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buf, &mut msg) == 0, "could not read iN!");

    let mut module = ptr::null_mut();

    assert!(LLVMParseIRInContext(context, memory_buf, &mut module, &mut msg) == 0, "Could not create module!");
    assert!(LLVMVerifyModule(module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut msg) == 0, "Could not validate!");

    // load function
    let fnc_name = CString::new("test").unwrap();
    let fnc = LLVMGetNamedFunction(module, fnc_name.as_ptr());

    assert!(fnc as usize != 0);

    use enzyme_sys::{createEmptyTypeAnalysis, AutoDiff, typeinfo::TypeInfo};
    let type_analysis = createEmptyTypeAnalysis();
    let auto_diff = AutoDiff::new(type_analysis);
    //pub fn create_primal_and_gradient(&self, context: *mut LLVMOpaqueContext, fnc_todiff: LLVMValueRef, ret_type: CDIFFE_TYPE, args: Vec<CDIFFE_TYPE>, type_info: typeinfo::TypeInfo) {
    auto_diff.create_primal_and_gradient(context as *mut enzyme_sys::LLVMOpaqueContext, fnc as *mut enzyme_sys::LLVMOpaqueValue, enzyme_sys::CDIFFE_TYPE::DFT_OUT_DIFF, Vec::new(), TypeInfo);

    panic!("");


    // get metadata
    let metadata = LLVMGetSubprogram(fnc);
    assert!(metadata as usize != 0, "Could not load metadata!");

    LLVMDisposeModule(module);
    LLVMContextDispose(context);
}

pub fn build() {
    pre_processing();
    //parser::parse();
    unsafe {load_llvm();}
}
