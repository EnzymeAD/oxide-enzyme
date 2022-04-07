#![allow(non_camel_case_types)]
mod foo;
use foo::j;
use oxide_enzyme::differentiate;
use simple_dep::{f, g};

// Enzyme will generally return structs (unless we choose the duplicated arguments convention).
// So let's define the return structs which we will use

/// Function f(x) = (x^2 + 2)^2 + 2x
/// Derivative f'(x) = 4x^3 + 8x + 2
/// f(1)  = 3^2+2=11
/// f'(1) = 4+8+2=14
//#[no_mangle]

// j(x) = x * x + 2

#[differentiate(d_test, Reverse, All(Active), Constant, false)]
fn test(x: f64) -> f64 {
    j(x) * j(x) + 2.0 * x
}

#[differentiate(d_test_ref, Reverse, All(Duplicated), None, false)]
fn test_ref(x: &mut f64) {
    *x = j(*x) + 2.0 * *x;
    // d_x = x + 2
}

#[no_mangle]
fn f_wrap(x: f64, y: f64) -> f64 {
    f(x, y)
}

#[no_mangle]
fn g_wrap(x: f64) -> f64 {
    g(x)
}

#[no_mangle]
fn h(x: f64, y: f64) -> f64 {
    2.0 * x + y
}

static mut X1: f64 = 0.0;
static mut D_X1: f64 = 0.0;
static mut X2: f64 = 1.0;
static mut D_X2: f64 = 0.0;

fn main() {
    unsafe {
        dbg!(d_test(1.0));
        println!("{} {}", X1, D_X1);
        dbg!(d_test_ref(&mut X1, &mut D_X1));
        println!("{} {}", X1, D_X1);

        println!("{} {}", X2, D_X2);
        dbg!(d_test_ref(&mut X2, &mut D_X2));
        println!("{} {}", X2, D_X2);
        //dbg!(&f_wrap(1.0, 1.0));
        //dbg!(&g_wrap(1.0));
        //dbg!(&enzyme3(1.0, 1.0));
        //dbg!(&multi_args4(2.0, 1.0, 1.0));
    }
}
