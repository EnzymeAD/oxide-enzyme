//extern crate llvm_sys;
//


use std::ptr;
use std::process;
use std::ffi::CString;
use std::ffi::CStr;

//use llvm_sys;
use llvm_sys::{core::*, debuginfo::LLVMGetSubprogram, execution_engine::LLVMCreateExecutionEngineForModule};
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction};
use llvm_sys::execution_engine::{LLVMGetFunctionAddress, LLVMLinkInMCJIT};
use llvm_sys::target::LLVM_InitializeNativeTarget;


use llvm_sys::*;
use llvm_sys::prelude::*;
use llvm_sys::core::*;
use llvm_sys::target_machine::*;
use llvm_sys::target::*;
use llvm_sys::analysis::LLVMVerifyFunction;

pub fn pre_processing() {
    //let out = process::Command::new("/usr/bin/rustc")
    let out = process::Command::new("/home/zuse/.cargo/bin/rustc")
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
    let fnc_name = CString::new("testx").unwrap();
    let fnc = LLVMGetNamedFunction(module, fnc_name.as_ptr());

    assert!(fnc as usize != 0);

    use enzyme_sys::{createEmptyTypeAnalysis, AutoDiff, typeinfo::TypeInfo};
    let type_analysis = createEmptyTypeAnalysis();
    let auto_diff = AutoDiff::new(type_analysis);

    let grad_func = auto_diff.create_primal_and_gradient(context as *mut enzyme_sys::LLVMOpaqueContext, fnc as *mut enzyme_sys::LLVMOpaqueValue, enzyme_sys::CDIFFE_TYPE::DFT_OUT_DIFF, Vec::new(), TypeInfo);

    //LLVMVerifyFunction(grad_func, &mut msg);

    println!("Function: {:?}", grad_func);
    //dbg!(&grad_func);

    LLVM_InitializeNativeTarget(); //needed for GetDefaultTriple()
    
    //LLVM_InitializeAllTargetInfos();
    //LLVM_InitializeAllTargetMCs();

    //LLVM_InitializeAllAsmParsers();
    LLVM_InitializeAllAsmPrinters(); // needed for LLVMTargetMachineEmitToFile()

    //let triple = CString::new("x86_64-unknown-linux-gnu").unwrap().into_raw();
    let triple = LLVMGetDefaultTargetTriple(); // works, TODO verify
    
    let mut target = ptr::null_mut();
    assert!(LLVMGetTargetFromTriple(triple, &mut target, &mut msg) == 0, "Could not get target machine from triple! {:?}", CStr::from_ptr(msg).to_str().unwrap());
    

    let cpu = LLVMGetHostCPUName();
    //let feature = "\0".as_ptr() as *const i8;
    let feature = LLVMGetHostCPUFeatures();
    println!("CPU: {:?}", CStr::from_ptr(cpu).to_str().unwrap());
    println!("Triple: {:?}", CStr::from_ptr(triple).to_str().unwrap());
    println!("Feature: {:?}", CStr::from_ptr(feature).to_str().unwrap());
    let opt_level = LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault;
    let reloc_mode = LLVMRelocMode::LLVMRelocDefault;
    let code_model = LLVMCodeModel::LLVMCodeModelDefault;
    let target_machine = LLVMCreateTargetMachine(target, triple, cpu, feature, opt_level, reloc_mode, code_model);
    assert!(!target_machine.is_null(), "target_machine is null!");
    println!("GOT TARGET MACHINE!");
    
    let dataLayout = LLVMCreateTargetDataLayout(target_machine);
    let dataLayoutStr = LLVMCopyStringRepOfTargetData(dataLayout);
    println!("DataLayout: {:?}", CStr::from_ptr(dataLayoutStr).to_str().unwrap());

    let file_type = LLVMCodeGenFileType::LLVMObjectFile;
    let output_file = CString::new("result.o").unwrap().into_raw();
    let output_txt = CString::new("result.txt").unwrap().into_raw();
    LLVMPrintModuleToFile(module, output_txt, &mut msg);
    assert!(LLVMTargetMachineEmitToFile(target_machine, module, output_file, file_type, &mut msg) == 0, "{:?}", CStr::from_ptr(msg).to_str().unwrap());
    
    //see: https://github.com/nagisa/llvm_build_utils.rs/blob/master/src/lib.rs#L463


    /*
     * pack to archive with https://docs.rs/cc/1.0.67/cc/struct.Build.html#method.compile */

    cc::Build::new()
      .object("result.o")
      .flag("-fPIE")
      .shared_flag(true)
      .static_flag(true)
      .compile("TestGrad");

    // emit link instruction \o/



    LLVMDisposeTargetMachine(target_machine);
    LLVMDisposeModule(module);
    LLVMContextDispose(context);
}

pub fn build() {
    pre_processing();
    //parser::parse();
    unsafe {load_llvm();}
}
