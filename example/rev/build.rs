#![allow(unused_variables)]
use oxide_enzyme::{FncInfo, ReturnActivity, CDIFFE_TYPE};
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../src/lib.rs");

    let entry_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let check_path = entry_path.join("enzyme-done");
    println!("cargo:rerun-if-changed={}", check_path.display());

    let reduce = FncInfo::new(
        "reduce_max",
        "d_reduce_max",
        vec![
            CDIFFE_TYPE::DFT_OUT_DIFF,
            CDIFFE_TYPE::DFT_CONSTANT,
            CDIFFE_TYPE::DFT_CONSTANT,
        ],
        ReturnActivity::Constant,
    );
    // #[differentiate(
    //     d_reduce,
    //     Reverse,
    //     PerInput(Duplicated, Constant, Constant),
    //     Constant,
    //     false
    // )]
    oxide_enzyme::build(vec![reduce]);
}
