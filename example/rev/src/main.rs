use approx::assert_relative_eq;
use oxide_enzyme::differentiate;

#[differentiate(
    d_reduce_max,
    Reverse,
    PerInput(Duplicated, Constant, Constant),
    Constant,
    //Active,
    false
    )]
fn reduce_max(ptr: *mut f64, length: usize, capacity: usize) -> f64 {
    let vec = unsafe { Vec::from_raw_parts(ptr, length, capacity) };
    // vec.into_iter()
    //     .fold(f64::MIN, |max, x| if x > max { max + x } else { max })
    let mut ret = f64::MIN;
    assert!(length < 6);
    for v in vec {
        if ret > v {
            ret = v;
        }
    }
    ret
}

fn main() {
    let len = 5;
    let mut input = vec![-1., 2., -0.2, 2., 1.];
    let capacity = input.capacity();
    let ptr = input.as_mut_ptr();
    let mut d_vec = vec![0.; len];
    let d_ptr = d_vec.as_mut_ptr();
    let max_val = reduce_max(ptr, len, capacity);
    println!("reduce_max: {max_val}");
    unsafe {
        d_reduce_max(ptr, d_ptr, len, capacity);
    }

    let ans: Vec<f64> = vec![0., 0., 0., 1., 0.];
    for i in 0..ans.len() {
        println!("{i} {} {}", ans[i], d_vec[i]);
        assert_relative_eq!(ans[i], d_vec[i]);
    }
}
