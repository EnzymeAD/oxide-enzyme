extern crate bindgen;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

//const LIBRARY_NAME: &'static str = "libenzyme.so"; // arch
const LIBRARY_NAME: &'static str = "LLVMEnzyme-11.so"; // Ubuntu

fn system_library(name: &str) -> Option<PathBuf> {
    // the Enzyme build script installs to /usr/local/lib
    fs::read_dir("/usr/local/lib/").unwrap()
        //.chain(fs::read_dir("source/enzyme/build/Enzyme").unwrap())
        //.chain(fs::read_dir("/usr/lib/").unwrap())
        .filter_map(|x| x.ok())
        .filter(|x| x.file_type().unwrap().is_file())
        .map(|x| x.path())
        .filter(|x| x.file_name().map(|x| x == name).unwrap_or(false))
        .next()
        .map(|x| x.parent().unwrap().to_owned())
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

fn generate_bindings() {
    let header_path = "source/enzyme/Enzyme/CApi.h";

    // tell cargo to re-run the builder if the header has changed
    println!("cargo:rerun-if-changed={}", header_path);

    let bindings = bindgen::Builder::default()
        .header(header_path)
        // add CConcreteType as enum
        .whitelist_type("CConcreteType")
        .rustified_enum("CConcreteType")
        .whitelist_type("CDIFFE_TYPE")
        .rustified_enum("CDIFFE_TYPE")
        .whitelist_type("LLVMContextRef")
        .whitelist_type("CTypeTreeRef")
        .whitelist_type("EnzymeTypeAnalysisRef")
        .whitelist_function("EnzymeNewTypeTree")
        .whitelist_function("EnzymeNewTypeTreeCT")
        .whitelist_function("EnzymeFreeTypeTree")
        .whitelist_function("EnzymeMergeTypeTree")
        .whitelist_function("EnzymeTypeTreeOnlyEq")
        .whitelist_function("EnzymeMergeTypeTree")
        .whitelist_function("EnzymeTypeTreeShiftIndiciesEq")
        .whitelist_function("EnzymeTypeTreeToString")
        .whitelist_function("EnzymeTypeTreeToStringFree")
        .whitelist_function("EnzymeGetGlobalAA")
        .whitelist_function("EnzymeFreeGlobalAA")
        .whitelist_type("LLVMOpaqueModule")
        //.whitelist_function("LLVMModuleCreateWithName")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("enzyme.rs"))
        .expect("Couldn't write bindings for enzyme!");
}

fn choose_library() {
    if let Some(path) = system_library(LIBRARY_NAME) {
        println!(
            "cargo:rustc-link-search={}",
            path.display()
        );
    } else {
        //panic!("");
        // create build folder
        let build_path = Path::new("source/enzyme/build");
        fs::create_dir_all(&build_path).unwrap();
            
        let mut cmake = Command::new("cmake");
        cmake
            .args(&["-G", "Ninja", "..", "-DLLVM_LIT=/home/zuse/Downloads/llvm-project-11.0.0/llvm/utils/lit/lit.py"])
            .current_dir(&build_path);

        run_and_printerror(&mut cmake);
            
        let mut ninja = Command::new("ninja");
        ninja
            .current_dir(&build_path);

        run_and_printerror(&mut ninja);
    }

    println!("cargo:rustc-link-lib=dylib=LLVMEnzyme-11");
    //println!("cargo:rustc-link-lib=dylib=enzyme");
    println!("cargo:rustc-link-lib=LLVM-11");
}

fn main() {
    generate_bindings();
    choose_library();
}
