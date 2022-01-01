#![allow(unused_variables)]
use oxide_enzyme::{FncInfo, CDIFFE_RETTYPE, CDIFFE_TYPE};
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../src/lib.rs");

    let entry_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let check_path = entry_path.join("enzyme-done");
    println!("cargo:rerun-if-changed={}", check_path.display());

    /*
    let fnc_1 = FncInfo::new("test","enzyme1",
                             vec![DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, true)));

    let fnc_2 = FncInfo::new("test","enzyme2",
                             vec![DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, false)));


    let fnc_4 = FncInfo::new("test","enzyme4",
                             vec![DFT_OUT_DIFF],
                             None);

    // Here we take the input as const and ignore the return,
    // so Enzyme won't return anything.
    let fnc_5 = FncInfo::new("test","enzyme5",
                             vec![DFT_CONSTANT],
                             None);

    // Not sure about this one
    let fnc_6 = FncInfo::new("test","enzyme6",
                             vec![DFT_DUP_ARG],
                             Some((DFT_OUT_DIFF, true)));

    let fnc_ref1 = FncInfo::new("test_ref","enzyme_ref",
                             vec![DFT_DUP_ARG],
                             Some((DFT_OUT_DIFF, false)));
    */
    let mult_1 = FncInfo::new(
        "h",
        "multi_args1",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF, CDIFFE_TYPE::DFT_CONSTANT],
        Some((CDIFFE_RETTYPE::DFT_OUT_DIFF, true)),
    );

    let mult_2 = FncInfo::new(
        "h",
        "multi_args2",
        vec![CDIFFE_TYPE::DFT_CONSTANT, CDIFFE_TYPE::DFT_OUT_DIFF],
        Some((CDIFFE_RETTYPE::DFT_OUT_DIFF, true)),
    );

    let mult_3 = FncInfo::new(
        "h",
        "multi_args3",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF, CDIFFE_TYPE::DFT_OUT_DIFF],
        Some((CDIFFE_RETTYPE::DFT_OUT_DIFF, false)),
    );

    let mult_4 = FncInfo::new(
        "h",
        "multi_args4",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF, CDIFFE_TYPE::DFT_OUT_DIFF],
        Some((CDIFFE_RETTYPE::DFT_OUT_DIFF, true)),
    );

    let fnc_3 = FncInfo::new(
        "test",
        "enzyme3",
        vec![CDIFFE_TYPE::DFT_OUT_DIFF],
        Some((CDIFFE_RETTYPE::DFT_OUT_DIFF, false)),
    );

    /*

    let fnc_ext1 = FncInfo::new("g_wrap","enzyme1",
                             vec![DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, true)));

    let fnc_ext2 = FncInfo::new("g_wrap","enzyme2",
                             vec![DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, false)));

    let fnc_ext3 = FncInfo::new("f_wrap","enzyme3",
                             vec![DFT_OUT_DIFF, DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, false)));

    let fnc_ext4 = FncInfo::new("f_wrap","enzyme4",
                             vec![DFT_OUT_DIFF,DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, true)));
    */

    oxide_enzyme::build(
        //vec![fnc_ext1, fnc_ext2, fnc_ext3]//, fnc_ext4]//, fnc_2, fnc_3, fnc_4, fnc_5, fnc_6]
        //vec![mult_1, mult_2, mult_3]
        vec![fnc_3, mult_4],
    );
}
