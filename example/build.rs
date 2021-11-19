use oxide_enzyme::{FncInfo, CDIFFE_TYPE::*};
use std::path::PathBuf;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../src/lib.rs");
 
    let entry_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let check_path = entry_path.join("enzyme-done");
    println!("cargo:rerun-if-changed={}", check_path.display());

    let fnc_1 = FncInfo::new("test","enzyme1",
                             vec![DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, true)));

    let fnc_2 = FncInfo::new("test","enzyme2",
                             vec![DFT_OUT_DIFF],
                             Some((DFT_OUT_DIFF, false)));

    let fnc_3 = FncInfo::new("test","enzyme3",
                             vec![DFT_CONSTANT],
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

    // Not sure about this one
    let fnc_ref1 = FncInfo::new("test_ref","enzyme_ref",
                             vec![DFT_DUP_ARG],
                             Some((DFT_OUT_DIFF, false)));

    let mult_1 = FncInfo::new("test_2","multi_args1",
                             vec![DFT_OUT_DIFF, DFT_CONSTANT],
                             None);

    let mult_2 = FncInfo::new("test_2","multi_args1",
                             vec![DFT_OUT_DIFF, DFT_CONSTANT],
                             Some((DFT_OUT_DIFF, true)));

    let mult_3 = FncInfo::new("test_2","multi_args1",
                             vec![DFT_OUT_DIFF, DFT_CONSTANT],
                             Some((DFT_OUT_DIFF, false)));



    oxide_enzyme::build(
        vec![fnc_1, fnc_2, fnc_3, fnc_4, fnc_5, fnc_6]
    );
}
