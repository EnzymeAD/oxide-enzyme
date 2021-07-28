use std::path::Path;
use dirs;

fn choose_library() {
    let platform = std::env::var("TARGET").unwrap();
    let enzyme_path = dirs::config_dir().unwrap().join("enzyme").join("Enzyme-0.0.16").join("enzyme").join("build").join("Enzyme");
    let llvm_path   = dirs::config_dir().unwrap().join("enzyme").join("rustc-1.54.0-src").join("build").join(&platform).join("llvm").join("lib");
    let enzyme_lib  = "Enzyme-12";
    let llvm_lib    = "LLVM-12-rust-1.56.0-nightly";
    assert!(enzyme_path.exists(), "enzyme dir couldn't be found: {}", enzyme_path.display());
    assert!(llvm_path.exists(),   "llvm dir couldn't be found: {}"  ,   llvm_path.display());
    println!("cargo:rustc-link-search={}", enzyme_path.display());
    println!("cargo:rustc-link-search={}",   llvm_path.display());
    println!("cargo:rustc-link-lib=dylib={}", llvm_lib); 
    println!("cargo:rustc-link-lib=dylib={}", enzyme_lib);
}

fn copy_bindings() {
    let cfg_dir = dirs::config_dir().expect("Enzyme needs access to your cfg dir.");
    let src = cfg_dir.join("enzyme").join("enzyme.rs");
    let dst = Path::new(&std::env::var("OUT_DIR").unwrap()).join("enzyme.rs");
    if !src.exists() { panic!("please first generate the bindings"); } 
    std::fs::copy(src,dst).expect("Copying over the bindings should never fail. Please submit a Bug report");
}

fn main() {
    copy_bindings();
    choose_library();
}
