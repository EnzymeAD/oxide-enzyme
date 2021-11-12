mod foo;
use foo::f;

/// Function f(x) = (x^2 + 2)^2 + 2x
/// Derivative f'(x) = 4x^3 + 8x + 2
#[no_mangle]
fn testx(x: f32) -> f32 {
    f(x) * f(x) + 2.0 * x
}


#[no_mangle]
fn test2(x: f32) -> f32 {
    2.0 * 2.0 * x
}

extern "C" {
    fn diffetest2(val: f32, differeturn: f32) -> f32;
}

#[repr(C)]
#[derive(Debug)]
struct Ret { first : f32, }
extern "C" {
    fn diffetestx(val: f32, differeturn: f32) -> Ret;
}

fn main() {
    unsafe { dbg!(&diffetestx(1.0, 1.0)); };
    let _foo = unsafe { diffetest2(1.0, 1.0) };
    println!("{}", testx(1.0));
}
