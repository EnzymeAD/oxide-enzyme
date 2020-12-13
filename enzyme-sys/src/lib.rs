#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/enzyme.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_tree() {
        let _ = unsafe {
            EnzymeNewTypeTree()
        };
    }

    #[test]
    fn build_tree() {
        //let tree = unsafe { EnzymeNewTypeTreeCT(

        let t = CConcreteType::DT_Float;

    }
}

