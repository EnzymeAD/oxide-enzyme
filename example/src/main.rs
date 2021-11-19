mod foo;
use foo::f;


// Enzyme will generally return structs (unless we choose the duplicated arguments convention).
// So let's define the return structs which we will use
#[repr(C)]
#[derive(Debug)]
struct Ret { first : f32 }
#[repr(C)]
#[derive(Debug)]
struct Ret2 { first : f32, second : f32 }
//#[repr(C)]
#[derive(Debug)]
struct Ret4 { first : f32, second : f32, third: f32, fourth: f32 }




/// Function f(x) = (x^2 + 2)^2 + 2x
/// Derivative f'(x) = 4x^3 + 8x + 2
/// f(1)  = 3^2+2=11
/// f'(1) = 4+8+2=14
#[no_mangle]
fn test(x: f32) -> f32 {
    f(x) * f(x) + 2.0 * x
}

#[no_mangle]
fn test_ref(x: &mut f32) {
    *x = f(*x) * f(*x) + 2.0 * *x;
}


// enzyme1 does return two floats, but if we try to access them both, 
// rustc will replace their type with a single double...
extern "C" {
    fn enzyme1(val: f32, differeturn: f32) -> Ret;
    fn enzyme2(val: f32, differeturn: f32) -> Ret;
    fn enzyme3(val: f32, differeturn: f32);
    fn enzyme4(val: f32, differeturn: f32) -> Ret;
    fn enzyme5(val: f32);
    fn enzyme6(val: f32, d_val: f32, differeturn: f32) -> Ret;

    //fn enzyme_ref(val: &f32, d_val: &f32);
}



static mut D_X: f32 = 0.0;
static mut X2: f32 = 1.0;
static mut D_X2: f32 = 0.0;


fn main() {
    unsafe {
        dbg!(&enzyme1(1.0, 1.0));
        dbg!(&enzyme2(1.0, 1.0));
        dbg!(&enzyme3(1.0, 1.0));
        dbg!(&enzyme4(1.0, 1.0));
        dbg!(&enzyme5(1.0));
        dbg!(&enzyme6(1.0, D_X, 2.0));
        println!("dx: {}", D_X);
        
        //dbg!(&enzyme_ref(&X2, &D_X2));
        println!("X2: {}, D_X2: {}", X2, D_X2);
    }
    dbg!(test(1.0));

}
