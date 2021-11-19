mod foo;
use foo::f;

/*
#[no_mangle]
fn testx(x: f32, y: f32) -> f32 {
    2.0 * x + y
}

#[repr(C)]
#[derive(Debug)]
struct Ret { first : f32, second : f32 }
extern "C" {
    fn diffetestx(x: f32, y: f32, differeturn1: f32, differeturn2: f32) -> Ret;
}
*/



/// Function f(x) = (x^2 + 2)^2 + 2x
/// Derivative f'(x) = 4x^3 + 8x + 2
/// f(1)  = 3^2+2=11
/// f'(1) = 4+8+2=14
#[no_mangle]
fn test(x: f32) -> f32 {
    f(x) * f(x) + 2.0 * x
}

#[repr(C)]
#[derive(Debug)]
struct Ret { first : f32 }


extern "C" {
    fn enzyme1(val: f32, differeturn: f32) -> Ret;
    fn enzyme2(val: f32, differeturn: f32) -> Ret;
    fn enzyme3(val: f32, differeturn: f32) -> Ret;
}





fn main() {
    unsafe { dbg!(&enzyme1(1.0, 1.0)); };
    unsafe { dbg!(&enzyme2(1.0, 1.0)); };
    unsafe { dbg!(&enzyme3(1.0, 1.0)); };
    //unsafe { dbg!(&enzyme3(1.0)); };
    dbg!(test(1.0));
}
