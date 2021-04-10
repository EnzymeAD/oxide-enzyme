// test.rs
extern { fn __enzyme_autodiff(_: usize, ...) -> f64; }

fn square(x : f64) -> f64 {
   return x * x;
}

fn main() {
   unsafe {
      println!("Hello, world {} {}!", square(3.0), __enzyme_autodiff(square as usize, 3.0));
   }
}
