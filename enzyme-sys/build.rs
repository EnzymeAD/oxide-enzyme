extern crate bindgen;
extern crate pkg_config;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn generate_bindings() {
    let header_path = format!("{}CApi.h", ENZYME_PATH);
    // tell cargo to re-run the builder if the header has changed
    println!("cargo:rerun-if-changed={}", header_path);

    let bindings = bindgen::Builder::default()
        .header(&header_path)
        // add CConcreteType as enum
        .whitelist_type("CConcreteType")
        .rustified_enum("CConcreteType")
        .whitelist_type("LLVMContextRef")
        .whitelist_function("EnzymeNewTypeTree")
        .whitelist_function("EnzymeNewTypeTreeCT")
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
    let enzyme_lib_path = Path::new("source")
        .join("enzyme")
        .join("build")
        .join("Enzyme");

        if !enzyme_lib_path.join("libllvmenzyme.so").exists() {
            // create make folder
            let conf_path = Path::new(ENZYME_PATH)
                .join("enzyme")
                .join("build");
            
            fs::create_dir_all(&conf_path).unwrap();
            
            let build_path = Path::new(ENZYME_PATH)
                .parent().unwrap().join("build");
            
            let cmake = Command::new("cmake")
                .args(&["-G", "Ninja", "..", "-DLLVM_DIR="])
                .arg("../../llvm/cmake/")
                .current_dir(&build_path)
                .output()
                .unwrap();
            
            dbg!(&cmake);
            
            let ninja = Command::new("ninja")
                .current_dir(&build_path)
                .output()
                .unwrap();
            
            dbg!(&ninja);
        }

        println!(
            "cargo:rustc-link-search={}",
            enzyme_lib_path.display()
        );

        println!("cargo:rustc-link-lib={}=enzyme", link_kind);
    }

}

fn main() {
    generate_bindings();
    choose_library();
}
