fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    oxide_enzyme::build();
}
