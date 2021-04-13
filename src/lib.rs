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




unsafe fn create_target_machine() -> LLVMTargetMachineRef {
    LLVM_InitializeNativeTarget(); //needed for GetDefaultTargetTriple()

    let triple = LLVMGetDefaultTargetTriple(); 
    let cpu = LLVMGetHostCPUName();
    let feature = LLVMGetHostCPUFeatures();
    let opt_level = LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault;
    let reloc_mode = LLVMRelocMode::LLVMRelocDefault;
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
    assert!(LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buf, &mut msg) == 0, "could not read iN!");

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
    //LLVMIsAFunction(grad_func);
    let context2 = LLVMContextCreate();

    // Create a LLVMModule with only grad_func inside for linking :)
    let mod_name = CString::new("gradModule").unwrap();
    let gradMod: LLVMModuleRef = LLVMModuleCreateWithName(mod_name.as_ptr());

    //LLVMAddFunction(gradMod, fnc_name.as_ptr() , grad_func);

    println!("TypeOf(grad_func) {:?}", LLVMTypeOf(grad_func));
    println!("param count: grad_func {:?}", LLVMCountParams(grad_func));
    //LLVMVerifyFunction(grad_func, &mut msg);

    println!("Function: {:?}", grad_func);

    let target_machine = create_target_machine(); // uses env Information to create a machine suitable for the user

    let output_file = CString::new("result.o").unwrap().into_raw();
    let output_txt = CString::new("result.txt").unwrap().into_raw();
    LLVMPrintModuleToFile(module, output_txt, &mut msg);
    //assert!(LLVMTargetMachineEmitToFile(target_machine, module, output_file, LLVMCodeGenFileType::LLVMObjectFile, &mut msg) == 0, "{:?}", CStr::from_ptr(msg).to_str().unwrap());
    

    let path = CString::new("./testx.bc").unwrap();
    let (context2, module2) = read_bc(path);
    assert!(LLVMTargetMachineEmitToFile(target_machine, module2, output_file, LLVMCodeGenFileType::LLVMObjectFile, &mut msg) == 0, "{:?}", CStr::from_ptr(msg).to_str().unwrap());
    
    //see: https://github.com/nagisa/llvm_build_utils.rs/blob/master/src/lib.rs#L463


    /*
     * pack to archive with https://docs.rs/cc/1.0.67/cc/struct.Build.html#method.compile */

    
    cc::Build::new()
      .object("result.o")
      //.flag("-fPIE")
      //.shared_flag(true)
      //.static_flag(true)
      .compile("TestGrad");
    
    //panic!("");

    // emit link instruction \o/



    LLVMDisposeMessage(msg);
    LLVMDisposeTargetMachine(target_machine);
    LLVMDisposeModule(module);
    LLVMContextDispose(context);
    LLVMContextDispose(context2);
}

pub fn build() {
    pre_processing();
    //parser::parse();
    unsafe {load_llvm();}
}
