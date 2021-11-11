#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/enzyme.rs"));
// TODO check where we should change the generated bindings and remove the mut. Apparently it's added everywhere (?), but enzyme handles quite a few args as const.
