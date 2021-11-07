use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::{env, process, ptr};

use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule};
use llvm_sys::core::*;
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use llvm_sys::LLVMLinkage;

use enzyme_sys::utils;
use enzyme_sys::{createEmptyTypeAnalysis, AutoDiff, EnzymeSetCLBool, SafeEnzymeSetCLBool};

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
fn compile_rs_to_bc(entry_file: &PathBuf, out_file: &PathBuf) {
    let rustc_path = utils::get_rustc_binary_path();
    let mut cmd = process::Command::new(rustc_path);
    cmd.args(&[
        "--emit=llvm-bc",
        "-O",
        &entry_file.to_str().unwrap(),
        "-g",
        "-o",
        &out_file.to_str().unwrap(),
    ]);

    run_and_printerror(&mut cmd);
}

/// Create target machine with default relocation/optimization/code model
unsafe fn create_target_machine() -> LLVMTargetMachineRef {
    LLVM_InitializeNativeTarget(); //needed for GetDefaultTargetTriple()

    LLVM_InitializeAllTargetInfos();
    LLVM_InitializeAllTargets();
    LLVM_InitializeAllTargetMCs();
    LLVM_InitializeAllAsmParsers();
    LLVM_InitializeAllAsmPrinters();

    let triple = LLVMGetDefaultTargetTriple();
    let cpu = LLVMGetHostCPUName();
    let feature = LLVMGetHostCPUFeatures();
    let opt_level = LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault; // TODO adjust
    let reloc_mode = LLVMRelocMode::LLVMRelocDynamicNoPic;
    let code_model = LLVMCodeModel::LLVMCodeModelDefault;

    println!("CPU: {:?}", CStr::from_ptr(cpu).to_str().unwrap());
    println!("Triple: {:?}", CStr::from_ptr(triple).to_str().unwrap());
    println!("Feature: {:?}", CStr::from_ptr(feature).to_str().unwrap());

    let mut msg = ptr::null_mut();

    // get target reference
    let mut target = ptr::null_mut();
    assert!(
        LLVMGetTargetFromTriple(triple, &mut target, &mut msg) == 0,
        "Could not get target machine from triple! {:?}",
        CStr::from_ptr(msg).to_str().unwrap()
    );

    // get target machine
    let target_machine = LLVMCreateTargetMachine(
        target, triple, cpu, feature, opt_level, reloc_mode, code_model,
    );
    assert!(!target_machine.is_null(), "target_machine is null!");

    LLVMDisposeMessage(msg);
    target_machine
}

/// Read the binary representation of LLVM IR code into a module and context
unsafe fn read_bc(path: &Path) -> (LLVMModuleRef, LLVMContextRef) {
    let context = LLVMContextCreate();
    let mut msg = ptr::null_mut();

    let path = CString::new(path.to_str().unwrap()).unwrap();
    let mut memory_buf = ptr::null_mut();
    assert_eq!(
        LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buf, &mut msg),
        0,
        "could not read in!"
    );

    let mut module = ptr::null_mut();

    assert!(
        LLVMParseIRInContext(context, memory_buf, &mut module, &mut msg) == 0,
        "Could not create module!"
    );
    assert!(
        LLVMVerifyModule(
            module,
            LLVMVerifierFailureAction::LLVMReturnStatusAction,
            &mut msg
        ) == 0,
        "Could not validate!"
    );

    LLVMDisposeMessage(msg);
    (module, context)
}

unsafe fn load_primary_functions(
    module: LLVMModuleRef,
    fnc_names: Vec<String>,
) -> Vec<LLVMValueRef> {
    let mut functions = vec![];
    for fnc in &fnc_names {
        let fnc_name = CString::new((*fnc).clone()).unwrap();
        let llvm_fnc = LLVMGetNamedFunction(module, fnc_name.as_ptr());
        assert_ne!(llvm_fnc as usize, 0, "couldn't find function {}", fnc);
        functions.push(llvm_fnc);
    }
    assert_eq!(
        functions.len(),
        fnc_names.len(),
        "load_llvm: couldn't find all functions!"
    );
    functions
}

unsafe fn generate_grad_function(
    context: LLVMContextRef,
    mut functions: Vec<LLVMValueRef>,
) -> Vec<LLVMValueRef> {
    let type_analysis = createEmptyTypeAnalysis();
    let auto_diff = AutoDiff::new(type_analysis);

    let mut grad_fncs = vec![];
    for &mut fnc in functions.iter_mut() {
        let grad_func: LLVMValueRef = auto_diff.create_primal_and_gradient(
            context as *mut enzyme_sys::LLVMOpaqueContext,
            fnc as *mut enzyme_sys::LLVMOpaqueValue,
            enzyme_sys::CDIFFE_TYPE::DFT_OUT_DIFF,
        ) as LLVMValueRef;
        grad_fncs.push(grad_func);
        println!("TypeOf(grad_func) {:?}", LLVMTypeOf(grad_func));
        println!("param count: grad_func {:?}", LLVMCountParams(grad_func));
        #[allow(deprecated)]
        let name = LLVMGetValueName(grad_func);
        println!("name: {:?}", name);
        println!("Function: {:?}", grad_func);
    }
    assert_eq!(
        grad_fncs.len(),
        functions.len(),
        "failed generating all gradient functions!"
    );
    grad_fncs
}

// Our Gradient fnc is returning a struct containing one element.
// Our Rust code expects a function returning the element, without the struct
// We create a new (identical) fnc which only differs in returning T rather than { T }.
// All it does is call enzyme's grad fnc and extract T from the struct, forwarding it.
unsafe fn extract_return_type(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    fnc: LLVMValueRef,
    u_type: LLVMTypeRef,
    f_type: LLVMTypeRef,
    fnc_name: String,
) -> LLVMValueRef {
    let param_num = LLVMCountParamTypes(LLVMGetElementType(f_type));
    let mut param_types: Vec<LLVMTypeRef> = vec![];
    param_types.reserve(param_num as usize);
    LLVMGetParamTypes(LLVMGetElementType(f_type), param_types.as_mut_ptr());
    let inner_fnc_name = "struct_".to_string() + &fnc_name;
    let c_inner_fnc_name = CString::new(inner_fnc_name.clone()).unwrap();
    let outer_fnc_name = fnc_name;
    let c_outer_fnc_name = CString::new(outer_fnc_name.clone()).unwrap();
    let new_fnc: LLVMValueRef = LLVMAddFunction(
        module,
        c_outer_fnc_name.as_ptr(),
        LLVMGetElementType(u_type) as LLVMTypeRef,
    );
    LLVMSetValueName2(
        fnc,
        c_inner_fnc_name.as_ptr(),
        inner_fnc_name.len() as usize,
    );

    let entry = "fnc_entry".to_string();
    let c_entry = CString::new(entry.clone()).unwrap();
    let basic_block = LLVMAppendBasicBlockInContext(context, new_fnc, c_entry.as_ptr());
    let mut fnc_args: Vec<LLVMValueRef> = vec![];
    fnc_args.reserve(param_num as usize);
    LLVMGetParams(new_fnc, fnc_args.as_mut_ptr());

    let builder = LLVMCreateBuilderInContext(context);
    LLVMPositionBuilderAtEnd(builder, basic_block);
    let struct_ret = LLVMBuildCall(
        builder,
        fnc,
        fnc_args.as_mut_ptr(),
        param_num,
        c_inner_fnc_name.as_ptr(),
    );
    let foo = "foo".to_string();
    let c_foo = CString::new(foo.clone()).unwrap();
    let struct_ret = LLVMBuildExtractValue(builder, struct_ret, 0, c_foo.as_ptr());
    let _ret = LLVMBuildRet(builder, struct_ret);
    let _terminator = LLVMGetBasicBlockTerminator(basic_block);
    //assert!(LLVMIsNull(terminator)!=0, "no terminator");
    LLVMDisposeBuilder(builder);

    assert!(
        LLVMVerifyFunction(new_fnc, LLVMVerifierFailureAction::LLVMAbortProcessAction) == 0,
        "Could not validate function!"
    );

    let mut msg = ptr::null_mut();
    assert!(
        LLVMVerifyModule(
            module,
            LLVMVerifierFailureAction::LLVMReturnStatusAction,
            &mut msg
        ) == 0,
        "Could not validate! {:?}",
        CStr::from_ptr(msg).to_str().unwrap()
    );
    LLVMDisposeMessage(msg);

    new_fnc
}

#[allow(non_snake_case)]
unsafe fn remove_U_symbols(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    grad_functions: &mut [LLVMValueRef],
    primary_fnc_names: Vec<String>,
) {
    for i in 0..grad_functions.len() {
        let name = &primary_fnc_names[i];

        // rename grad fnc to tmp name (to not hide equally named undef symbols anymore)
        let tmp = "tmp_diffe".to_owned() + &name;
        let c_tmp = CString::new(tmp.clone()).unwrap();
        LLVMSetValueName2(grad_functions[i], c_tmp.as_ptr(), tmp.len() as usize);

        // access undef symbols
        let new_fnc_name: String = "diffe".to_owned() + &name;
        let c_fnc_name = CString::new(new_fnc_name.clone()).unwrap();
        let u_fnc: LLVMValueRef = LLVMGetNamedFunction(module, c_fnc_name.as_ptr()); // get the U(ndefined) fnc symbol
        assert_ne!(
            u_fnc as usize, 0,
            "couldn't get undef symbol {}",
            new_fnc_name
        );

        let u_type: LLVMTypeRef = LLVMTypeOf(u_fnc);
        let f_type: LLVMTypeRef = LLVMTypeOf(grad_functions[i]);
        let u_type_string = CString::from_raw(LLVMPrintTypeToString(u_type.clone()));
        let f_type_string = CString::from_raw(LLVMPrintTypeToString(f_type.clone()));
        let u_return_type = LLVMGetReturnType(LLVMGetElementType(u_type));
        let f_return_type = LLVMGetReturnType(LLVMGetElementType(f_type));

        if u_type != f_type {
            // Type mismatch which we should fix
            if u_return_type == f_return_type {
                panic!("Return types match. However a different, unhandled missmatch occured: u: {:?}, f: {:?}", u_type_string, f_type_string);
            }

            // TODO: What if return type isn't a struct? e.g. by using a different enzyme style
            if LLVMCountStructElementTypes(f_return_type) != 1 {
                panic!(
                    "Return struct contains more than one element. u: {:?}, f: {:?}",
                    u_type_string, f_type_string
                );
            }
            grad_functions[i] = extract_return_type(
                module,
                context,
                grad_functions[i],
                u_type,
                f_type,
                new_fnc_name.clone(),
            );
        }

        // Clean up
        LLVMReplaceAllUsesWith(u_fnc, grad_functions[i]);
        LLVMDeleteFunction(u_fnc);
        LLVMSetValueName2(
            grad_functions[i],
            c_fnc_name.as_ptr(),
            new_fnc_name.len() as usize,
        );
    }
}

unsafe fn dumb_module_to_obj(module: LLVMModuleRef, context: LLVMContextRef, out_obj: &Path) {
    let target_machine = create_target_machine();
    let mut msg = ptr::null_mut();
    let c_out_obj = CString::new(out_obj.to_str().unwrap().to_owned())
        .unwrap()
        .into_raw();
    assert!(
        LLVMTargetMachineEmitToFile(
            target_machine,
            module,
            c_out_obj,
            LLVMCodeGenFileType::LLVMObjectFile,
            &mut msg
        ) == 0,
        "filename: {:?}, error: {:?}",
        out_obj,
        CStr::from_ptr(msg).to_str().unwrap()
    );
    LLVMDisposeMessage(msg);
    LLVMDisposeTargetMachine(target_machine);
    LLVMDisposeModule(module);
    LLVMContextDispose(context);
}

unsafe fn localize_all_symbols(module: LLVMModuleRef) {
    let mut symbol = LLVMGetFirstFunction(module);
    let last_symbol = LLVMGetLastFunction(module);
    if symbol == last_symbol {
        panic!("Found no symbols in module");
    }
    while symbol != last_symbol {
        LLVMSetLinkage(symbol, LLVMLinkage::LLVMInternalLinkage);
        symbol = LLVMGetNextFunction(symbol);
    }
}
unsafe fn globalize_grad_symbols(module: LLVMModuleRef, primary_fnc_names: Vec<String>) {
    for primary_name in primary_fnc_names {
        let grad_fnc_name = "diffe".to_owned() + &primary_name;
        let c_grad_fnc_name = CString::new(grad_fnc_name.clone()).unwrap();
        let grad_fnc = LLVMGetNamedFunction(module, c_grad_fnc_name.as_ptr());
        assert_ne!(
            grad_fnc as usize, 0,
            "couldn't find function {}",
            grad_fnc_name
        );
        LLVMSetLinkage(grad_fnc, LLVMLinkage::LLVMExternalLinkage);
    }
}

pub fn build<T: AsRef<Path>>(entry_file: T, primary_fnc_names: Vec<String>) {
    let mut entry_path = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    entry_path.push(entry_file.as_ref().file_name().unwrap());
    let out_obj = entry_path
        .clone()
        .with_file_name("result")
        .with_extension("o");
    let out_bc = entry_path
        .clone()
        .with_file_name("result")
        .with_extension("bc");

    compile_rs_to_bc(&entry_file.as_ref().to_path_buf(), &out_bc);

    unsafe {

        let (module, context) = read_bc(&out_bc);
        let functions = load_primary_functions(module, primary_fnc_names.clone());
        SafeEnzymeSetCLBool(true);
        let mut grad_fncs = generate_grad_function(context, functions);
        SafeEnzymeSetCLBool(false);
        remove_U_symbols(module, context, &mut grad_fncs, primary_fnc_names.clone());
        localize_all_symbols(module);
        globalize_grad_symbols(module, primary_fnc_names);
        dumb_module_to_obj(module, context, &out_obj);
    };

    // compile to static archive
    cc::Build::new().object(out_obj).compile("GradFunc");
}
