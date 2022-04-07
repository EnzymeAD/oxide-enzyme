use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::{env, fs, ptr};

use llvm_sys::core::*;
use llvm_sys::ir_reader::LLVMParseIRInContext;
use llvm_sys::prelude::*;
use llvm_sys::target::*;
use llvm_sys::target_machine::*;
use llvm_sys::LLVMLinkage;

use glob::glob;
use std::process::Command;

pub use autodiff::differentiate_ext as differentiate;

#[doc(hidden)]
mod enzyme;
#[doc(hidden)]
mod verify;
#[doc(hidden)]
mod wrappers;
pub use enzyme::{enzyme_print_activity, enzyme_print_functions, enzyme_print_type};
use enzyme::{AutoDiff, LLVMOpaqueValue, ParamInfos};
pub use enzyme::{FncInfo, ReturnActivity, CDIFFE_TYPE};

fn llvm_bin_dir() -> PathBuf {
    let rustc_ver = env!("RUSTC_VER");
    let target = env!("TARGET");
    dirs::cache_dir()
        .unwrap()
        .join("enzyme")
        .join("rustc-".to_owned() + rustc_ver + "-src")
        .join("build")
        .join(target)
        .join("llvm")
        .join("build")
        .join("bin")
}

fn llvm_objcopy() -> PathBuf {
    llvm_bin_dir().join("llvm-objcopy")
}
fn llvm_link() -> PathBuf {
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
fn create_target_machine() -> LLVMTargetMachineRef {
    let (triple, cpu, feature) = unsafe {
        LLVM_InitializeNativeTarget(); //needed for GetDefaultTargetTriple()

        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllAsmParsers();
        LLVM_InitializeAllAsmPrinters();

        let triple = LLVMGetDefaultTargetTriple();
        let cpu = LLVMGetHostCPUName();
        let feature = LLVMGetHostCPUFeatures();
        (triple, cpu, feature)
    };

    let opt_level = if cfg!(debug_assertions) {
        LLVMCodeGenOptLevel::LLVMCodeGenLevelNone
    } else {
        LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive
    };

    let reloc_mode = LLVMRelocMode::LLVMRelocPIC;

    // https://doc.rust-lang.org/rustc/codegen-options/index.html#code-model
    let code_model = LLVMCodeModel::LLVMCodeModelSmall;

    unsafe {
        dbg!("CPU:", CStr::from_ptr(cpu).to_str().unwrap());
        dbg!("Triple:", CStr::from_ptr(triple).to_str().unwrap());
        dbg!("Feature:", CStr::from_ptr(feature).to_str().unwrap());
    }

    let mut target = ptr::null_mut();
    let mut msg = ptr::null_mut();

    unsafe {
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
}

pub(crate) fn get_type(t: LLVMTypeRef) -> CString {
    unsafe { CString::from_raw(LLVMPrintTypeToString(t)) }
}

/// Read the binary representation of LLVM IR code into a module and context
fn read_bc_files(fnc_names: Vec<String>) -> (LLVMModuleRef, LLVMContextRef) {
    // Collect some environment information
    let crate_name: String = env::var("CARGO_PKG_NAME").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let merged_bc = out_dir
        .join("merged.bc")
        .into_os_string()
        .into_string()
        .unwrap();

    let central_dir = out_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let deps_dir = central_dir.join("deps");

    let mut bc_files: Vec<String> = vec![];
    let mut main_bc: String = "".to_owned();
    let search_term = deps_dir.join("*.bc");
    let search_results = glob(search_term.to_str().unwrap())
        .expect("Failed to read glob pattern. Please report this!");
    for path in search_results.flatten() {
        let bc_string_name = path.into_os_string().into_string().unwrap();
        if bc_string_name.starts_with(deps_dir.join(&crate_name).to_str().unwrap()) {
            main_bc = bc_string_name;
        } else {
            bc_files.push(bc_string_name);
        }
    }
    dbg!(&bc_files);
    assert_ne!(
        "", main_bc,
        "Couldn't find central bc file. Please report this!"
    );

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

    unsafe {
        let context = LLVMContextCreate();
        let mut msg = ptr::null_mut();

        let path = CString::new(merged_bc).unwrap();
        let mut memory_buf = ptr::null_mut();
        assert_eq!(
            LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), &mut memory_buf, &mut msg),
            0,
            "Could not read LLVM-bc file. Please report this!"
        );

        let mut module = ptr::null_mut();

        assert!(
            LLVMParseIRInContext(context, memory_buf, &mut module, &mut msg) == 0,
            "Could not create the module. Please report this!"
        );
        LLVMDisposeMessage(msg);

        if let Err(e) = verify::verify_module(module) {
            panic!(
                "The input module was already broken. Please report this! {}",
                e
            );
        }

        (module, context)
    }
}

fn load_primary_functions(module: LLVMModuleRef, fnc_names: Vec<String>) -> Vec<LLVMValueRef> {
    let mut functions: Vec<LLVMValueRef> = vec![];
    for fnc_name in &fnc_names {
        let c_name = CString::new(fnc_name.clone()).unwrap();
        let llvm_fnc = unsafe { LLVMGetNamedFunction(module, c_name.as_ptr()) };
        assert_ne!(
            llvm_fnc as usize, 0,
            "We couldn't find the function definition for {}. Please add it.",
            fnc_name
        );
        functions.push(llvm_fnc);
    }
    assert_eq!(
        functions.len(),
        fnc_names.len(),
        "We couldn't find all functions, that's a bug. Please report it!"
    );
    functions
}

fn generate_grad_function(
    mut functions: Vec<LLVMValueRef>,
    grad_names: Vec<String>,
    mut param_infos: Vec<ParamInfos>,
) -> Vec<LLVMValueRef> {
    let opt_grads = !cfg!(debug_assertions); // There should be a better solution
    let auto_diff = AutoDiff::new(opt_grads);

    let mut grad_fncs = vec![];
    for (&mut fnc, (param_info, grad_name)) in functions
        .iter_mut()
        .zip(param_infos.iter_mut().zip(grad_names.iter()))
    {
        dbg!(grad_name);
        let grad_func: LLVMValueRef = auto_diff.create_primal_and_gradient(
            fnc as *mut LLVMOpaqueValue,
            &mut param_info.input_activity,
            param_info.ret_info,
        ) as LLVMValueRef;
        dbg!("Generated gradient function");
        grad_fncs.push(grad_func);
        let llvm_grad_fnc_type = unsafe { LLVMTypeOf(grad_func) };
        dbg!(get_type(llvm_grad_fnc_type));
        dbg!(unsafe { LLVMCountParams(grad_func) });
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
        "We failed generating all gradient functions. Please report it!"
    );
    grad_fncs
}

fn print_ffi_type(module: LLVMModuleRef, ffi_names: Vec<String>) {
    unsafe {
        for name in ffi_names {
            let c_fnc_name = CString::new(name.clone()).unwrap();
            let u_fnc: LLVMValueRef = LLVMGetNamedFunction(module, c_fnc_name.as_ptr()); // get the U(ndefined) fnc symbol
            assert_ne!(u_fnc as usize, 0, "couldn't get undef symbol {}", name);

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
fn handle_ffi(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    grad_functions: &mut [LLVMValueRef],
    grad_names: Vec<String>,
) {
    for i in 0..grad_functions.len() {
        let grad_name = &grad_names[i];

        // rename grad fnc to tmp name (to not hide equally named undef symbols anymore)
        let tmp = "tmp_diffe".to_owned() + grad_name;
        let c_tmp = CString::new(tmp.clone()).unwrap();
        unsafe {
            LLVMSetValueName2(grad_functions[i], c_tmp.as_ptr(), tmp.len() as usize);
        }

        // access undef symbols
        let c_fnc_name = CString::new(grad_name.clone()).unwrap();
        // get the U(ndefined) fnc symbol
        let u_fnc: LLVMValueRef = unsafe { LLVMGetNamedFunction(module, c_fnc_name.as_ptr()) };
        assert_ne!(
            u_fnc as usize, 0,
            "Couldn't get undef symbol for {}. \
                   Do you declare your gradient function in an extern block?",
            grad_name
        );

        unsafe {
            let u_type: LLVMTypeRef = LLVMTypeOf(u_fnc);
            let f_type: LLVMTypeRef = LLVMTypeOf(grad_functions[i]);
            let u_return_type = LLVMGetReturnType(LLVMGetElementType(u_type));
            let f_return_type = LLVMGetReturnType(LLVMGetElementType(f_type));

            if u_type != f_type {
                dbg!("Some type missmatch happened for ".to_owned() + &grad_names[i]);
                dbg!(get_type(u_type));
                dbg!(get_type(f_type));
                dbg!(get_type(u_return_type));
                dbg!(get_type(f_return_type));
                dbg!();

                // TODO: Check for 2xf32 -> 1xf64 changes
                let num_elem_in_ret_struct = LLVMCountStructElementTypes(f_return_type);

                if num_elem_in_ret_struct > 2 {
                    dbg!("move_return_into_args");
                    // The C-Abi will change a function returning a struct with more than
                    // two double values by returning void and moving the actual return struct
                    // into the parameter list, at the first position.
                    grad_functions[i] = wrappers::move_return_into_args(
                        module,
                        context,
                        grad_functions[i],
                        u_type,
                        grad_name.clone(),
                    );
                } else if num_elem_in_ret_struct == 1 {
                    dbg!("extract_return_type");
                    // The C-Abi will change a function returning a struct { double } with exactly
                    // one double value to just return the double, stripping the struct.
                    grad_functions[i] = wrappers::extract_return_type(
                        module,
                        context,
                        grad_functions[i],
                        u_type,
                        grad_name.clone(),
                    );
                } else {
                    panic!("Unhandled type missmatch. Please report this.");
                }
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
}

fn dumb_module_to_obj(module: LLVMModuleRef, context: LLVMContextRef, out_obj: &Path) {
    unsafe {
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
}

fn only_expose_gradients(module: LLVMModuleRef, fncs: Vec<LLVMValueRef>) {
    unsafe {
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

    for grad_fnc in fncs {
        unsafe {
            LLVMSetLinkage(grad_fnc, LLVMLinkage::LLVMExternalLinkage);
        }
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
            res.push(symbol);
            symbol = LLVMGetNextFunction(symbol);
        }
        res
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
            if let Err(e) = verify::verify_function(fnc) {
                panic!("Creating a wrapper function failed! {}", e);
            }

            // LLVMDeleteFunction(fnc); // Breaks other things
            // TODO: Something like LLVMVerifyFunction(fnc);
            // or probably better LLVMVerifyModule
        }
    }
}

fn build_archive(primary_fnc_infos: Vec<FncInfo>) {
    let entry_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_obj = entry_path.with_file_name("result").with_extension("o");
    let out_archive = entry_path
        .join("libGradFunc.a")
        .into_os_string()
        .into_string()
        .unwrap();

    // Let's split it up so we can just pass those values which ufnction need.
    let (mut primary_names, mut grad_names, mut parameter_informations) = (vec![], vec![], vec![]);
    for info in primary_fnc_infos.clone() {
        primary_names.push(info.primary_name);
        grad_names.push(info.grad_name);
        parameter_informations.push(info.params);
    }

    // Merge and load the bitcode files with some care to have all the code which we might differentiate
    let (module, context) = read_bc_files(primary_names.clone());

    // Store existing functions name to clean up later
    let junk_fnc = list_functions(module);

    // Just for debugging purpose, some type infos
    print_ffi_type(module, grad_names.clone());

    // We are loading the existing primary functions, to pass them to enzyme.
    let functions = load_primary_functions(module, primary_names.clone());

    if let Err(e) = verify::verify_user_inputs(primary_fnc_infos, functions.clone(), context) {
        panic!("The primary function which you wrote does not work with the FncInfo which you gave! {}", e);
    }

    // Now we generate the gradients based on our input and the selected activity values for
    // their parameters
    enzyme_print_type(cfg!(debug_assertions)); // print generated functions in debug mode
    let mut grad_fncs =
        generate_grad_function(functions, grad_names.clone(), parameter_informations);
    enzyme_print_type(false);

    // Now that we have the gradients, lets clean up
    remove_functions(junk_fnc);

    // First, some magic to handle ffi
    handle_ffi(module, context, &mut grad_fncs, grad_names.clone());

    // The next step breaks some module rules, but is necessary to not multiple symbol definitions.
    // So we check our module for other issues before.
    if let Err(e) = unsafe { verify::verify_module(module) } {
        panic!(
            "Apparently we broke the module while working on it. Please report this! {}",
            e
        );
    }

    // Next, we localize all other symbols, since we only want to expose the newly generated functions
    only_expose_gradients(module, grad_fncs);

    // And now we store all gradients in a single object file
    dumb_module_to_obj(module, context, &out_obj);

    // compile object file to static archive
    cc::Build::new().object(out_obj).compile("GradFunc");

    // And remove the extra __rust_probestack
    // https://github.com/rust-lang/rust/issues/88274
    let mut objcopy = Command::new(llvm_objcopy());
    objcopy.args(&[
        "--localize-symbol",
        "__rust_probestack",
        &out_archive,
        &out_archive,
    ]);
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
