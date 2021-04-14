use std::{ptr, process, env};
use std::ffi::{CString, CStr};
use std::path::{Path, PathBuf};

use llvm_sys::core::*;
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::analysis::{LLVMVerifyModule, LLVMVerifierFailureAction};
use llvm_sys::target::LLVM_InitializeNativeTarget;
use llvm_sys::prelude::*;
use llvm_sys::target_machine::*;
use llvm_sys::target::*;

use enzyme_sys::{createEmptyTypeAnalysis, AutoDiff, typeinfo::TypeInfo};

/// Run a command and panic with error message if not succeeded
fn run_and_printerror(command: &mut process::Command) {
    match command.status() {
        Ok(status) => {
            if !status.success() {
                panic!("Failed: `{:?}` ({})", command, status);
            }
        }
        Err(error) => {
            panic!("Failed: `{:?}` ({})", command, error);
        }
    }
}

/// Generate LLVM BC artifact
///
/// Compiles entry point into LLVM IR binary representation with debug informations. The artifact
/// is used to generate the derivative function with Enzyme.
pub fn generate_bc(entry_file: &Path) -> PathBuf {
    let mut out_file = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    out_file.push(entry_file.file_name().unwrap());

    let mut cmd = process::Command::new("/usr/bin/rustc");
    //let mut cmd = process::Command::new("/home/zuse/.cargo/bin/rustc")
    cmd.args(&["--emit=llvm-bc",  "src/main.rs", "-C", "debuginfo=2", "-o", &out_file.to_str().unwrap()]);

    run_and_printerror(&mut cmd);

    out_file
}


/// Create target machine with default relocation/optimization/code model
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

    // get target reference
    let mut target = ptr::null_mut();
    assert!(
        LLVMGetTargetFromTriple(triple, &mut target, &mut msg) == 0, 
        "Could not get target machine from triple! {:?}", CStr::from_ptr(msg).to_str().unwrap()
    );

    // get target machine
    let target_machine = LLVMCreateTargetMachine(target, triple, cpu, feature, opt_level, reloc_mode, code_model);
    assert!(!target_machine.is_null(), "target_machine is null!");
    
    LLVMDisposeMessage(msg);
    target_machine
}

/// Read the binary representation of LLVM IR code into a module and context
unsafe fn read_bc(path: &Path) -> (LLVMContextRef, LLVMModuleRef) {
    let context = LLVMContextCreate();
    let mut msg = ptr::null_mut();

    let path = CString::new(path.to_str().unwrap()).unwrap();
    let mut memory_buf = ptr::null_mut();
    assert!(LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buf, &mut msg) == 0, "could not read in!");

    let mut module = ptr::null_mut();

    assert!(LLVMParseIRInContext(context, memory_buf, &mut module, &mut msg) == 0, "Could not create module!");
    assert!(LLVMVerifyModule(module, LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut msg) == 0, "Could not validate!");

    LLVMDisposeMessage(msg);
    (context, module)
}


pub unsafe fn load_llvm(bc_file: &Path, fnc: &str) -> (LLVMContextRef, LLVMModuleRef, LLVMValueRef) {
    LLVM_InitializeAllAsmPrinters(); // needed for LLVMTargetMachineEmitToFile()

    // load generated artifact
    let (context, module) = read_bc(bc_file);

    // load function
    let fnc_name = CString::new(fnc).unwrap();
    let fnc = LLVMGetNamedFunction(module, fnc_name.as_ptr());
    assert!(fnc as usize != 0);
    
    (context, module, fnc)
}

unsafe fn generate_grad_function(context: LLVMContextRef, fnc: LLVMValueRef) {
    let type_analysis = createEmptyTypeAnalysis();
    let auto_diff = AutoDiff::new(type_analysis);

    let grad_func: LLVMValueRef = auto_diff.create_primal_and_gradient(context as *mut enzyme_sys::LLVMOpaqueContext, fnc as *mut enzyme_sys::LLVMOpaqueValue, enzyme_sys::CDIFFE_TYPE::DFT_OUT_DIFF, Vec::new(), TypeInfo) as LLVMValueRef;

    println!("TypeOf(grad_func) {:?}", LLVMTypeOf(grad_func));
    println!("param count: grad_func {:?}", LLVMCountParams(grad_func));
    println!("Function: {:?}", grad_func);
}

unsafe fn emit_obj(module: LLVMModuleRef, entry_file: &Path) -> PathBuf {
    let target_machine = create_target_machine(); // uses env Information to create a machine suitable for the user

    let (out, out_stripped, out_text) = (
        entry_file.with_file_name("result").with_extension("o"),
        entry_file.with_file_name("result_stripped").with_extension("o"),
        entry_file.with_file_name("result").with_extension("txt")
    );

    let mut msg = ptr::null_mut();
    let output_file = CString::new(out.to_str().unwrap()).unwrap().into_raw();
    let output_txt = CString::new(out_text.to_str().unwrap()).unwrap().into_raw();
    LLVMPrintModuleToFile(module, output_txt, &mut msg);

    assert!(LLVMTargetMachineEmitToFile(target_machine, module, output_file, LLVMCodeGenFileType::LLVMObjectFile, &mut msg) == 0, "{:?}", CStr::from_ptr(msg).to_str().unwrap());

    // objcopy result.o result_stripped.o --globalize-symbol=diffetestx --keep-symbol=diffetestx --redefine-sym diffetestx.1=diffetestx -S
    let mut objcopy_cmd = std::process::Command::new("objcopy");
    out.args(&[
        out, &out_stripped,
        "--globalize-symbol=diffetestx",
        .arg("--globalize-symbol=diffetestx")
        .arg("--keep-symbol=diffetestx")
        .arg("--redefine-sym").arg("diffetestx.1=diffetestx")
        .arg("-S")
        .output().unwrap();
    
    LLVMDisposeMessage(msg);
    LLVMDisposeTargetMachine(target_machine);

    out_stripped
}

pub fn build(entry_file: &Path, fnc: &str) {
    let bc_file = generate_bc(entry_file);
    let out_stripped = unsafe {
        let (context, module, fnc) = load_llvm(&bc_file, fnc);
        generate_grad_function(context, fnc);
        let obj = emit_obj(module, entry_file);

        LLVMDisposeModule(module);
        LLVMContextDispose(context);

        obj
    };

    // compile to static archive
    cc::Build::new()
      .object(out_stripped)
      .compile("GradFunc");
}
