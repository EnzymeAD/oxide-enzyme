use oxide_enzyme::crate_type;
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/main.rs");
    println!("cargo:rerun-if-changed=../src/lib.rs");
    
    oxide_enzyme::build(
        vec![crate_type::bin],
        vec!["testx".to_owned(),"test2".to_owned() ]
    );
}
