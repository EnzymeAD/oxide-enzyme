use oxide_enzyme::crate_type;
use std::path::{Path, PathBuf};
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../src/lib.rs");
 
    let entry_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let check_path = entry_path.join("enzyme-done");
    println!("cargo:rerun-if-changed={}", check_path.display());

    oxide_enzyme::build(
        vec!["testx".to_owned(),"test2".to_owned() ]
    );
}
