use std::path::Path;

const ENZYME_VER: &str = "0.0.24";
const RUSTC_VER: &str = "1.57.0";
const LLVM_VER: &str = "13";

fn choose_library() {
    let platform = std::env::var("TARGET").unwrap();
    let enzyme_basedir = dirs::cache_dir().unwrap().join("enzyme");
    let enzyme_path = enzyme_basedir.join("Enzyme-".to_owned() + ENZYME_VER).join("enzyme").join("build").join("Enzyme");
    let llvm_path   = enzyme_basedir.join("rustc-".to_owned() + RUSTC_VER + "-src").join("build").join(&platform).join("llvm").join("lib");
    let enzyme_lib  = "Enzyme-".to_owned() + LLVM_VER;
    let llvm_lib    = "LLVM-".to_owned() + LLVM_VER + "-rust-" + RUSTC_VER + "-nightly";
    assert!(enzyme_path.exists(), "enzyme dir couldn't be found: {}", enzyme_path.display());
    assert!(llvm_path.exists(),   "llvm dir couldn't be found: {}"  ,   llvm_path.display());
    println!("cargo:rustc-link-search={}", enzyme_path.display());
    println!("cargo:rustc-link-search={}",   llvm_path.display());
    println!("cargo:rustc-link-lib=dylib={}", llvm_lib); 
    println!("cargo:rustc-link-lib=dylib={}", enzyme_lib);
}

fn copy_bindings() {
    let cache_dir = dirs::cache_dir().expect("Enzyme needs access to your cache dir.");
    let src = cache_dir.join("enzyme").join("enzyme.rs");
    let dst = Path::new(&std::env::var("OUT_DIR").unwrap()).join("enzyme.rs");
    if !src.exists() { panic!("please first generate the bindings"); } 
    std::fs::copy(src,dst).expect("Copying over the bindings should never fail. Please submit a Bug report");
}

fn main() {
    println!(
        "cargo:rustc-env=RUSTC_VER={}",
        RUSTC_VER
    );
    println!(
        "cargo:rustc-env=TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    copy_bindings();
    choose_library();
}
