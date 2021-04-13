//extern crate llvm_sys;
//


use std::ptr;
use std::process;
use std::ffi::CString;
use std::ffi::CStr;

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
    let out = process::Command::new("/usr/bin/rustc")
    //let out = process::Command::new("/home/zuse/.cargo/bin/rustc")
        .args(&["--emit=obj", "--emit=llvm-bc",  "src/main.rs", "-C", "debuginfo=2"])
        .output()
        .expect("failed to run cargo");

    dbg!(&out);
}


unsafe fn create_target_machine() -> LLVMTargetMachineRef {
    LLVM_InitializeNativeTarget(); //needed for GetDefaultTargetTriple()

    let triple = LLVMGetDefaultTargetTriple(); 
    let cpu = LLVMGetHostCPUName();
    let feature = LLVMGetHostCPUFeatures();
    let opt_level = LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault;
    let reloc_mode = LLVMRelocMode::LLVMRelocDynamicNoPic;
    let code_model = LLVMCodeModel::LLVMCodeModelDefault;
    
    println!("CPU: {:?}", CStr::from_ptr(cpu).to_str().unwrap());
    println!("Triple: {:?}", CStr::from_ptr(triple).to_str().unwrap());
    println!("Feature: {:?}", CStr::from_ptr(feature).to_str().unwrap());

    let mut msg = ptr::null_mut();
    // Create Target Machine
    let mut target = ptr::null_mut();
    assert!(LLVMGetTargetFromTriple(triple, &mut target, &mut msg) == 0, "Could not get target machine from triple! {:?}", CStr::from_ptr(msg).to_str().unwrap());
    let target_machine = LLVMCreateTargetMachine(target, triple, cpu, feature, opt_level, reloc_mode, code_model);
    assert!(!target_machine.is_null(), "target_machine is null!");
    println!("Got TargetMachine!");
    
    /*
    let dataLayout = LLVMCreateTargetDataLayout(target_machine);
    let dataLayoutStr = LLVMCopyStringRepOfTargetData(dataLayout);
    println!("DataLayout: {:?}", CStr::from_ptr(dataLayoutStr).to_str().unwrap());
    */

    LLVMDisposeMessage(msg);
    target_machine
}


unsafe fn read_bc(path: CString) -> (LLVMContextRef, LLVMModuleRef) {
    let context = LLVMContextCreate();
    let mut msg = ptr::null_mut();

    let mut memory_buf = ptr::null_mut();
    assert!(LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buf, &mut msg) == 0, "could not read in!");

    let mut module = ptr::null_mut();

    assert!(LLVMParseIRInContext(context, memory_buf, &mut module, &mut msg) == 0, "Could not create module!");
    assert!(LLVMVerifyModule(module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut msg) == 0, "Could not validate!");

    LLVMDisposeMessage(msg);
    (context, module)
}


pub unsafe fn load_llvm() {
    LLVM_InitializeAllAsmPrinters(); // needed for LLVMTargetMachineEmitToFile()

    let path = CString::new("./main.bc").unwrap();
    let (context, module) = read_bc(path);

    // load function
    let fnc_name = CString::new("testx").unwrap();
    let fnc = LLVMGetNamedFunction(module, fnc_name.as_ptr());
    
    let mut msg = ptr::null_mut();

    assert!(fnc as usize != 0);

    use enzyme_sys::{createEmptyTypeAnalysis, AutoDiff, typeinfo::TypeInfo};
    let type_analysis = createEmptyTypeAnalysis();
    let auto_diff = AutoDiff::new(type_analysis);

    let grad_func: LLVMValueRef = auto_diff.create_primal_and_gradient(context as *mut enzyme_sys::LLVMOpaqueContext, fnc as *mut enzyme_sys::LLVMOpaqueValue, enzyme_sys::CDIFFE_TYPE::DFT_OUT_DIFF, Vec::new(), TypeInfo) as LLVMValueRef;

    //LLVMAddFunction(newModule, fnc_name.as_ptr(), grad_func); // TODO Not that simple, but test as a cleaner alternative ?

    println!("TypeOf(grad_func) {:?}", LLVMTypeOf(grad_func));
    println!("param count: grad_func {:?}", LLVMCountParams(grad_func));
    println!("Function: {:?}", grad_func);

    let target_machine = create_target_machine(); // uses env Information to create a machine suitable for the user

    let output_file = CString::new("result.o").unwrap().into_raw();
    let output_txt = CString::new("result.txt").unwrap().into_raw();
    LLVMPrintModuleToFile(module, output_txt, &mut msg);

    assert!(LLVMTargetMachineEmitToFile(target_machine, module, output_file, LLVMCodeGenFileType::LLVMObjectFile, &mut msg) == 0, "{:?}", CStr::from_ptr(msg).to_str().unwrap());

    // objcopy result.o result_stripped.o --globalize-symbol=diffetestx --keep-symbol=diffetestx --redefine-sym diffetestx.1=diffetestx -S
    let out = std::process::Command::new("objcopy")
        .arg("result.o").arg("result_stripped.o").arg("-w")
        .arg("--globalize-symbol=diffe*")
        .arg("--globalize-symbol=augmented_*")
        .arg("--globalize-symbol=preprocess_*")
        .arg("--localize-symbol=*")
        //.arg("--keep-symbol=diffe*")
        //.arg("--keep-symbol=augmented_*")
        //.arg("--keep-symbol=preprocess_*")
        .arg("--keep-symbol=*")
        .arg("--redefine-sym").arg("diffetestx.1=diffetestx")
        //.arg("--redefine-sym").arg("diffef.2=diffef")
        .arg("-S")
        .output().unwrap();
    
    cc::Build::new()
      .object("result_stripped.o")
      .compile("TestGrad");
    
    LLVMDisposeMessage(msg);
    LLVMDisposeTargetMachine(target_machine);
    LLVMDisposeModule(module);
    LLVMContextDispose(context);
    //LLVMContextDispose(context2);
}

pub fn build() {
    pre_processing();
    unsafe {load_llvm();}
}
