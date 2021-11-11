use dirs;
use std::path::{Path, PathBuf};

const RUSTC_VER: &str = "1.56.0";


pub fn get_rustc_binary_path() -> PathBuf {
    let cfg_dir = dirs::config_dir().expect("Enzyme needs access to your cfg dir.");
    let platform = std::env::var("TARGET").unwrap();
    let rustc_path = cfg_dir
        .join("enzyme")
        .join("rustc-".to_owned() + RUSTC_VER + "-src")
        .join("build")
        .join(&platform)
        .join("stage2")
        .join("bin")
        .join("rustc");
    if !Path::exists(&rustc_path) {
        panic!("We use a custom rustc build which was expected to exist at {}\n
               Please have a look at enzyme_build.",rustc_path.display());
    }
    rustc_path
}
