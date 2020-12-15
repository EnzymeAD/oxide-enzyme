//extern crate llvm_sys;
//

mod parser;

use std::ptr;
use std::process;
use std::ffi::CString;

use llvm_sys::{core::*, debuginfo::LLVMGetSubprogram, execution_engine::LLVMCreateExecutionEngineForModule};
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction};
use llvm_sys::execution_engine::{LLVMGetFunctionAddress, LLVMLinkInMCJIT};
use llvm_sys::target::LLVM_InitializeNativeTarget;

pub fn pre_processing() {
    process::Command::new("/usr/bin/rustc")
        .args(&["--emit=obj", "--emit=llvm-bc",  "src/main.rs", "-C debuginfo=2"])
        .output()
        .expect("failed to run cargo");
}

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0").as_ptr() as *const i8
    );
}

pub unsafe fn load_llvm() {
    let context = LLVMContextCreate();
    let mut msg = ptr::null_mut();

    let mut memory_buf = ptr::null_mut();
    let path = CString::new("./main.bc").unwrap();
    if LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buf, &mut msg) != 0 {
        panic!("Could not read iN!");
    }

    let mut module = ptr::null_mut();
    if LLVMParseIRInContext(context, memory_buf, &mut module, &mut msg) != 0 {
        panic!("Could not create module!");
    }

    if LLVMVerifyModule(module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut msg) != 0{
        panic!("Could not validate!");
    }

    // load function
    let fnc_name = CString::new("fnc").unwrap();
    let fnc = LLVMGetNamedFunction(module, fnc_name.as_ptr());

    if fnc as usize == 0 {
        panic!("blub");
    }

    // get metadata
    let metadata = LLVMGetSubprogram(fnc);
    if metadata as usize == 0 {
        panic!("Could not load metadata");
    }

    LLVMDisposeModule(module);
    LLVMContextDispose(context);
}

pub fn build() {
    pre_processing();
    parser::parse();
    unsafe {load_llvm();}
}
