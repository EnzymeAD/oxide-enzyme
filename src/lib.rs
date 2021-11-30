use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::{fs, env, ptr};

use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule};
use llvm_sys::core::*;
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use llvm_sys::LLVMLinkage;

use glob::glob;
use std::process::Command;

mod enzyme;
use enzyme::{create_empty_type_analysis, AutoDiff, enzyme_print_type, enzyme_print_functions, enzyme_print_activity};
use enzyme::{LLVMOpaqueValue, ParamInfos};
pub use enzyme::{CDIFFE_TYPE, FncInfo};
use dirs;

fn llvm_bin_dir() -> PathBuf {
    dirs::cache_dir().unwrap()
        .join("enzyme")
        .join("rustc-1.56.0-src")
        .join("build")
        .join("x86_64-unknown-linux-gnu")
        .join("llvm")
        .join("build")
        .join("bin")
}

pub fn llvm_objcopy() -> PathBuf {
    llvm_bin_dir().join("llvm-objcopy")
}
pub fn llvm_link() -> PathBuf {
    llvm_bin_dir().join("llvm-link")
}

fn run_and_printerror(command: &mut Command) {
    println!("Running: `{:?}`", command);
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

    let opt_level = if cfg!(debug_assertions) {
        LLVMCodeGenOptLevel::LLVMCodeGenLevelNone
    } else {
        LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive
    };
    
    let reloc_mode = LLVMRelocMode::LLVMRelocPIC;

    // https://doc.rust-lang.org/rustc/codegen-options/index.html#code-model
    let code_model = LLVMCodeModel::LLVMCodeModelSmall;

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

fn get_type(t: LLVMTypeRef) -> CString {
    unsafe {
        CString::from_raw(LLVMPrintTypeToString(t))
    }
}

/// Read the binary representation of LLVM IR code into a module and context
unsafe fn read_bc_files(fnc_names: Vec<String>) -> (LLVMModuleRef, LLVMContextRef) {

    // Collect some environment information
    let crate_name: String = env::var("CARGO_PKG_NAME").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let merged_bc = out_dir.join("merged.bc").into_os_string().into_string().unwrap();

    let central_dir = out_dir
        .parent().unwrap()
        .parent().unwrap()
        .parent().unwrap();
    let deps_dir = central_dir.join("deps");

    let mut bc_files: Vec<String> = vec![];
    let mut main_bc: String = "".to_owned();
    let search_term = deps_dir.join("*.bc");
    let search_results = glob(search_term.to_str().unwrap()).expect("Failed to read glob pattern");
    for entry in search_results {
        if let Ok(path) = entry {
            let bc_string_name = path.into_os_string().into_string().unwrap();
            if bc_string_name.starts_with(deps_dir.join(&crate_name).to_str().unwrap()) {
                main_bc = bc_string_name;
            } else {
                bc_files.push(bc_string_name);
            }
        }
    }
    dbg!(&bc_files);
    assert_ne!("", main_bc, "Couldn't find central bc file");

    let mut merge = Command::new(&llvm_link());
    merge.current_dir(&central_dir);
    for fnc in fnc_names {
        merge.args(&["--import", &fnc, &main_bc]);
    }
    for bc in bc_files {
        merge.arg(&bc);
    }
    merge.args(&["--only-needed", "-o", &merged_bc]);
    run_and_printerror(&mut merge);

    let context = LLVMContextCreate();
    let mut msg = ptr::null_mut();

    let path = CString::new(merged_bc).unwrap();
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
    let mut functions: Vec<LLVMValueRef> = vec![];
    for fnc_name in &fnc_names {
        let c_name = CString::new(fnc_name.clone()).unwrap();
        let llvm_fnc = LLVMGetNamedFunction(module, c_name.as_ptr());
        assert_ne!(llvm_fnc as usize, 0, "couldn't find {}", fnc_name);
        functions.push(llvm_fnc);
    }
    assert_eq!(
        functions.len(),
        fnc_names.len(),
        "load_llvm: couldn't find all functions!"
    );
    // panic!("foo {} {}", functions.len(), fnc_names.len());
    functions
}

unsafe fn generate_grad_function(
    mut functions: Vec<LLVMValueRef>,
    grad_names: Vec<String>,
    mut param_infos: Vec<ParamInfos>,
) -> Vec<LLVMValueRef> {
    let type_analysis = create_empty_type_analysis();
    let auto_diff = AutoDiff::new(type_analysis);

    let mut grad_fncs = vec![];
    let opt_grads = if cfg!(debug_assertions) { false } else { true };
    for (&mut fnc, (param_info, grad_name)) in functions.iter_mut().zip(param_infos.iter_mut().zip(grad_names.iter())) {
        dbg!(grad_name);
        let grad_func: LLVMValueRef = auto_diff.create_primal_and_gradient(
            fnc as *mut LLVMOpaqueValue,
            &mut param_info.input_activity,
            param_info.ret_info,
            opt_grads
        ) as LLVMValueRef;
        dbg!("Generated gradient function");
        grad_fncs.push(grad_func);
        let llvm_grad_fnc_type = LLVMTypeOf(grad_func);
        dbg!(get_type(llvm_grad_fnc_type));
        dbg!(LLVMCountParams(grad_func));
        /*
        #[allow(deprecated)]
        let llvm_name = LLVMGetValueName(grad_func);
        let name = CString::from_raw(llvm_name as *mut i8);
        dbg!(name);*/
        dbg!(grad_func);
        dbg!();
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
    dbg!("Unpacking", fnc_name.clone());
    dbg!("From: ", get_type(f_type), " into ", get_type(u_type));
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

fn print_ffi_type(module: LLVMModuleRef, ffi_names: Vec<String>) {
    unsafe {
        for name in ffi_names {
            let c_fnc_name = CString::new(name.clone()).unwrap();
            let u_fnc: LLVMValueRef = LLVMGetNamedFunction(module, c_fnc_name.as_ptr()); // get the U(ndefined) fnc symbol
            assert_ne!(
                u_fnc as usize, 0,
                "couldn't get undef symbol {}",
                name
            );

            let u_type: LLVMTypeRef = LLVMTypeOf(u_fnc);
            let u_return_type = LLVMGetReturnType(LLVMGetElementType(u_type));

            dbg!("Expected type for ", name);
            dbg!(get_type(u_type));
            dbg!(get_type(u_return_type));
            dbg!();
        }
    }
}

#[allow(non_snake_case)]
unsafe fn remove_U_symbols(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    grad_functions: &mut [LLVMValueRef],
    grad_names: Vec<String>,
    primary_names: Vec<String>,
) {
    for i in 0..grad_functions.len() {

        let grad_name = &grad_names[i];

        // rename grad fnc to tmp name (to not hide equally named undef symbols anymore)
        let tmp = "tmp_diffe".to_owned() + &grad_name;
        let c_tmp = CString::new(tmp.clone()).unwrap();
        LLVMSetValueName2(grad_functions[i], c_tmp.as_ptr(), tmp.len() as usize);

        // access undef symbols
        let c_fnc_name = CString::new(grad_name.clone()).unwrap();
        let u_fnc: LLVMValueRef = LLVMGetNamedFunction(module, c_fnc_name.as_ptr()); // get the U(ndefined) fnc symbol
        assert_ne!(
            u_fnc as usize, 0,
            "couldn't get undef symbol {}",
            grad_name
        );

        let u_type: LLVMTypeRef = LLVMTypeOf(u_fnc);
        let f_type: LLVMTypeRef = LLVMTypeOf(grad_functions[i]);
        let u_return_type = LLVMGetReturnType(LLVMGetElementType(u_type));
        let f_return_type = LLVMGetReturnType(LLVMGetElementType(f_type));

        let u_type_string = CString::from_raw(LLVMPrintTypeToString(u_type.clone()));
        let f_type_string = CString::from_raw(LLVMPrintTypeToString(f_type.clone()));
        let u_ret_type_string = CString::from_raw(LLVMPrintTypeToString(u_return_type.clone()));
        let f_ret_type_string = CString::from_raw(LLVMPrintTypeToString(f_return_type.clone()));

        if u_type != f_type {
            dbg!("Some type missmatch happened for ".to_owned()+&grad_names[i]);
            dbg!(u_type_string); 
            dbg!(f_type_string);
            dbg!(u_ret_type_string);
            dbg!(f_ret_type_string);
            dbg!();
            // Type mismatch which we should fix
            /*
            if u_return_type == f_return_type {
                panic!("Return types match. However a different, unhandled missmatch occured: u: {:?}, f: {:?}", u_type_string, f_type_string);
            }
            */

            /*
            // TODO: What if return type isn't a struct? e.g. by using a different enzyme style
            // ANSWER: I guess it's fine?
            if LLVMCountStructElementTypes(f_return_type) != 1 {
                panic!(
                    "Return struct contains more than one element. u: {:?}, f: {:?}",
                    u_type_string, f_type_string
                );
            }
            */

            grad_functions[i] = extract_return_type(
                module,
                context,
                grad_functions[i],
                u_type,
                f_type,
                grad_name.clone(),
            );
        }

        // Clean up
        LLVMReplaceAllUsesWith(u_fnc, grad_functions[i]);
        LLVMDeleteFunction(u_fnc);
        LLVMSetValueName2(
            grad_functions[i],
            c_fnc_name.as_ptr(),
            grad_name.len() as usize,
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

    // All functions
    let mut symbol = LLVMGetFirstFunction(module);
    let last_symbol = LLVMGetLastFunction(module);
    while symbol != last_symbol {
        LLVMSetLinkage(symbol, LLVMLinkage::LLVMInternalLinkage);
        symbol = LLVMGetNextFunction(symbol);
    }
    LLVMSetLinkage(symbol, LLVMLinkage::LLVMInternalLinkage);

    // Global Symbols
    let mut symbol = LLVMGetFirstGlobal(module);
    let last_symbol = LLVMGetLastGlobal(module);
    while symbol != last_symbol {
        LLVMSetLinkage(symbol, LLVMLinkage::LLVMInternalLinkage);
        symbol = LLVMGetNextGlobal(symbol);
    }
    LLVMSetLinkage(symbol, LLVMLinkage::LLVMInternalLinkage);
}
unsafe fn globalize_grad_symbols(module: LLVMModuleRef, grad_fnc_names: Vec<String>) {
    for grad_fnc_name in grad_fnc_names {
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

fn verify_argument_len(functions: &Vec<LLVMValueRef>, 
                       fnc_names: Vec<String>, grad_names: Vec<String>, 
                       activity_vecs: Vec<ParamInfos>) {

    assert_eq!(functions.len(), fnc_names.len(), "Programmer bug, please report this message on Github");
    assert_eq!(functions.len(), grad_names.len(), "Programmer bug, please report this message on Github");
    assert_eq!(functions.len(), activity_vecs.len(), "Programmer bug, please report this message on Github");

    for i in 0..functions.len() {
        let fnc = functions[i];
        let act = &activity_vecs[i];
        let primary_fnc_name = &fnc_names[i];
        let grad_fnc_name = &grad_names[i];

        let num_primary_parameters;
        unsafe {
            num_primary_parameters = LLVMCountParams(fnc);
        }
        let num_activity_infos = act.input_activity.len() as u32;
        assert_eq!(num_primary_parameters, num_activity_infos, "Missmatch while generating {} from function {}.
            Please specify exactly one activity (CDIFFE_TYPE) value 
            for each of the input parameters of your primary function.", grad_fnc_name, primary_fnc_name);
    }
}

fn list_functions(module: LLVMModuleRef) -> Vec<LLVMValueRef> {
    unsafe {
        let mut res = vec![];
        let mut symbol = LLVMGetFirstFunction(module);
        let last_symbol = LLVMGetLastFunction(module);
        if symbol == last_symbol {
            panic!("Found no symbols in module");
        }
        while symbol != last_symbol {
            res.push(symbol.clone());
            symbol = LLVMGetNextFunction(symbol);
        }
        return res;
    }
}

fn remove_functions(fncs: Vec<LLVMValueRef>) {
    unsafe {
        for fnc in fncs {
            let num = LLVMCountBasicBlocks(fnc);
            let mut bb: Vec<LLVMBasicBlockRef> = Vec::with_capacity(num as usize);
            LLVMGetBasicBlocks(fnc, bb.as_mut_ptr());
            for block in bb {
                LLVMDeleteBasicBlock(block);
                // LLVMRemoveBasicBlockFromParent(block);
            }
            // LLVMDeleteFunction(fnc); // Breaks other things
        }
    }
}

fn build_archive(primary_fnc_infos: Vec<FncInfo>) {
    let entry_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_obj = entry_path
        .clone()
        .with_file_name("result")
        .with_extension("o");
    let out_archive = entry_path
        .clone()
        .join("libGradFunc.a")
        .into_os_string()
        .into_string()
        .unwrap();
    

    // Most functions will only require one, so let's split it up.
    let (mut primary_names, mut grad_names, mut parameter_informations) = (vec![], vec![], vec![]);
    for info in primary_fnc_infos {
        primary_names.push(info.primary_name);
        grad_names.push(info.grad_name);
        parameter_informations.push(info.params);
    }


    unsafe {

        // Merge and load the bitcode files with some care to have all the code which we might differentiate
        let (module, context) = read_bc_files(primary_names.clone());

        // Store existing functions name to clean up later
        let junk_fnc = list_functions(module);

        // Just for debugging purpose, some type infos
        print_ffi_type(module, grad_names.clone());

        // We are loading the existing primary functions, to pass them to enzyme.
        let functions = load_primary_functions(module, primary_names.clone());
        
        // Enzyme might deduce some things, but lets make it explicit. One activity information for
        // each input parameter.
        verify_argument_len(&functions, primary_names.clone(), grad_names.clone(), parameter_informations.clone());

        //enzyme_print_type(cfg!(debug_assertions)); // print generated functions in debug mode
        enzyme_print_type(true); // print generated functions in debug mode

        // Now we generate the gradients based on our input and the selected activity values for
        // their parameters
        let mut grad_fncs = generate_grad_function(functions, grad_names.clone(), parameter_informations);
        //enzyme_print_type(false);

        // Now that we have the gradients, lets clean up
        // remove_functions(junk_fnc);

        // Some magic to make the symbols link together nicely
        remove_U_symbols(module, context, &mut grad_fncs, grad_names.clone(), primary_names.clone());
        localize_all_symbols(module);
        globalize_grad_symbols(module, grad_names);

        // And now we store all gradients in a single object file
        dumb_module_to_obj(module, context, &out_obj);
    };

    // compile to static archive
    cc::Build::new().object(out_obj).compile("GradFunc");
    
    let mut objcopy = Command::new(llvm_objcopy());
    objcopy.args(&["--localize-symbol", "__rust_probestack", &out_archive, &out_archive]);
    run_and_printerror(&mut objcopy);
}


/// 
pub fn build(primary_functions: Vec<FncInfo>) {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let control_file = out_path.join("enzyme-done");

    if Path::exists(&control_file) {
        dbg!("second call"); // now we create and link the archive from the .bc file
        dbg!();
        fs::remove_file(&control_file).unwrap();
        build_archive(primary_functions);
        println!("cargo:rustc-link-search={}", out_path.display()); // cc does that already afaik
        println!("cargo:rustc-link-lib=static=GradFunc"); // cc does that already afaik
    } else {
        dbg!("first call"); // now cargo/rustc will generate the .bc file
        fs::File::create(control_file).unwrap();
    }
}
