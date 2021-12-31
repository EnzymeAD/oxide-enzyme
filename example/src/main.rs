mod foo;
use foo::j;
use simple_dep::{f,g};

// Enzyme will generally return structs (unless we choose the duplicated arguments convention).
// So let's define the return structs which we will use
#[repr(C)]
#[derive(Debug)]
struct Ret1 { first : f64 }
#[repr(C)]
#[derive(Debug)]
struct Ret2 { first : f64, second : f64 }
#[repr(C)]
#[derive(Debug)]
struct Ret3 { first : f64, second : f64, third: f64 }
#[repr(C)]
#[derive(Debug)]
struct Ret4 { first : f64, second : f64, third: f64, fourth: f64 }




/// Function f(x) = (x^2 + 2)^2 + 2x
/// Derivative f'(x) = 4x^3 + 8x + 2
/// f(1)  = 3^2+2=11
/// f'(1) = 4+8+2=14
#[no_mangle]
fn test(x: f64) -> f64 {
    j(x) * j(x) + 2.0 * x
}

#[no_mangle]
fn test_ref(x: &mut f64) {
    *x = j(*x) * j(*x) + 2.0 * *x;
}

#[no_mangle]
fn f_wrap(x: f64, y: f64) -> f64 {
    f(x,y)
}

#[no_mangle]
fn g_wrap(x: f64) -> f64 {
    g(x)
}

#[no_mangle]
fn h(x: f64, y: f64) -> f64 {
    2.0*x+y
}


// enzyme1 does return two floats, but if we try to access them both, 
// rustc will replace their type with a single double...
extern "C" {
    fn multi_args4( _: f64, _: f64, _: f64 ) -> Ret3;
    fn multi_args1( _: f64, _: f64, _: f64 ) -> Ret2;
    fn multi_args2( _: f64, _: f64, _: f64 ) -> Ret2;
    fn     enzyme3( _: f64, _: f64 ) -> f64;
    //fn multi_args3( _: f64, _: f64, _:f64 ) -> Ret2;
    // fn enzyme1( _:f64, _:f64 ) -> Ret2;
    // fn enzyme2( _: f64, _:f64) -> Ret;
    /*
    fn enzyme4( _: f64, _:f64, _:f64) -> Ret3;
    fn enzyme5(val: f64);
    fn enzyme6(val: f64, d_val: f64, differeturn: f64) -> Ret;
    */

    //fn enzyme_ref(val: &f64, d_val: &f64);
    //fn enzyme_f(x: f64, y: f64) -> Ret2;
}



static mut D_X: f64 = 0.0;
static mut X2: f64 = 1.0;
static mut D_X2: f64 = 0.0;


fn main() {
    unsafe {
        //dbg!(&f_wrap(1.0, 1.0));
        dbg!(&g_wrap(1.0));
        dbg!(&enzyme3(1.0, 1.0));
        //dbg!(&multi_args1(2.0, 1.0, 1.0));
        //dbg!(&multi_args2(2.0, 1.0, 1.0));
        //dbg!(&multi_args3(2.0, 1.0, 1.0));
        dbg!(&multi_args4(2.0, 1.0, 1.0));
        /*
        dbg!(&enzyme4(1.0, 1.0, 1.0));
        dbg!(&enzyme5(1.0));
        dbg!(&enzyme6(1.0, D_X, 2.0));
        println!("dx: {}", D_X);
        */
        //dbg!(&enzyme_ref(&X2, &D_X2));
        //println!("X2: {}, D_X2: {}", X2, D_X2);
        //dbg!(&enzyme_f(1.0, 1.0));
    }
    //dbg!(test(1.0));

}
