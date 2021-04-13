#[no_mangle]
fn testx(x: f32) -> f32 {
    x * x * x * x
}

extern "C" {
    fn diffetestx(val: f32, differeturn: f32) -> f32;
}

fn main() {
    unsafe { dbg!(&diffetestx(10.0, 1.0)); };
}

