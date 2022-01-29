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

    let mult_1 = FncInfo::new(
        "h",
        "multi_args1",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF, CDIFFE_TYPE::DFT_CONSTANT],
        ReturnActivity::Active,
    );

    let mult_2 = FncInfo::new(
        "h",
        "multi_args2",
        vec![CDIFFE_TYPE::DFT_CONSTANT, CDIFFE_TYPE::DFT_OUT_DIFF],
        ReturnActivity::Active,
    );

    let mult_3 = FncInfo::new(
        "h",
        "multi_args3",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF, CDIFFE_TYPE::DFT_OUT_DIFF],
        ReturnActivity::Gradient,
    );

    let mult_4 = FncInfo::new(
        "h",
        "multi_args4",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF, CDIFFE_TYPE::DFT_OUT_DIFF],
        ReturnActivity::Active,
    );

    let fnc_3 = FncInfo::new(
        "test",
        "enzyme3",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF],
        ReturnActivity::Gradient,
    );

    let d_test = FncInfo::new(
        "test",
        "d_test",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF],
        ReturnActivity::Constant,
        //ReturnActivity::Active, // returns {f64,f64}
        //ReturnActivity::Gradient, // returns {f64}
    );

    let d_test_ref = FncInfo::new(
        "test_ref",
        "d_test_ref",
        vec![CDIFFE_TYPE::DFT_DUP_ARG],
        ReturnActivity::None,
        //ReturnActivity::Active, // returns {f64,f64}
        //ReturnActivity::Gradient, // returns {f64}
    );

    oxide_enzyme::build(vec![d_test_ref]);
}
