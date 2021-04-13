#[no_mangle]
fn f(x: f32) -> f32 {
    x * x + 1.0
}

/// Function f(x) = (x^2 + 1)^2 + 2x
/// Derivative f'(x) = 4x^3 + 4x + 2
#[no_mangle]
fn testx(x: f32) -> f32 {
    f(x) * f(x) + 2.0 * x
}

extern "C" {
    fn diffetestx(val: f32, differeturn: f32) -> f32;
}

fn main() {
    unsafe { dbg!(&diffetestx(1.0, 1.0)); };
}

