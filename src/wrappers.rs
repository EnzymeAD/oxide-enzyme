use crate::get_type;
use crate::verify::{compare_param_types, verify_function};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ffi::CString;

/// This function creates and returns a wrapper function 'fnc_name' around the given function.
///
/// The wrapped function is expected to return a struct with three or more f64 values.
/// The wrapper function will accept the same arguments as the wrapped function,
/// except of an extra struct as the first argument. The wrapper will pass all other
/// arguments to the wrapped function and update the extra struct parameter based on
/// the return value of the wrapped function.
///
/// # Safety
///
/// The `module`, `context`, and `fnc` must all be valid.
/// The function `fnc` must be part of the given module and return a struct with three or more f64
/// values and no other content.
/// `u_type` and LLVMTypeOf(fnc) shall only differ by the position of the struct, `u_type` must
/// therefore return void ( () on Rust level).
pub unsafe fn move_return_into_args(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    fnc: LLVMValueRef,
    u_type: LLVMTypeRef,
    fnc_name: String,
) -> LLVMValueRef {
    let f_type = LLVMTypeOf(fnc);
    dbg!("Moving", fnc_name.clone());
    dbg!("From: ", get_type(f_type), " into ", get_type(u_type));

    let inner_param_num = LLVMCountParams(fnc);
    let (outer_fnc, outer_bb, mut outer_args, inner_args, c_inner_fnc_name) =
        create_wrapper(module, context, fnc, u_type, fnc_name);

    let _inner_ret_type = LLVMGetReturnType(LLVMGetElementType(f_type));
    let outer_ret_type = LLVMGetReturnType(LLVMGetElementType(u_type));
    if outer_ret_type != LLVMVoidTypeInContext(context) {
        let is = CString::from_raw(LLVMPrintTypeToString(outer_ret_type));
        let should = CString::from_raw(LLVMPrintTypeToString(LLVMVoidType()));
        panic!(
            "Return struct isn't moved into args. Please report this. {} vs. {}",
            is.to_str().unwrap(),
            should.to_str().unwrap()
        );
    }

    if outer_args.len() != 1 + inner_param_num as usize {
        panic!(
            "Outer wrapper should have exactly one extra arg. Please report this. {} vs {}",
            inner_param_num,
            outer_args.len()
        )
    }

    let mut input_args = outer_args.split_off(1);
    let _out_extra_arg = LLVMTypeOf(outer_args[0]);

    /*
     * TODO: Find out how to fix this check
    // the out_extra_arg might be a user-specified struct. We'll look up it's name
    // and use the name to look up it's actual definition, to compare it.
    //let out_type_name = LLVMGetStructName(out_extra_arg);
    if inner_ret_type != out_extra_arg {
        dbg!(43);
        //let inner_ret = get_type(inner_ret_type);
        let inner_ret = get_type(inner_ret_type);
        let extra_arg = get_type(out_extra_arg);
        let foo = LLVMGetTypeByName2(context, &extra_arg);
        panic!("Ret of inner should be identical to first param of outer. Please report this. {:?} vs. {:?}. Name: {:?}",
               inner_ret, extra_arg, 42);
    }
    */
    if let Err(e) = compare_param_types(input_args.clone(), inner_args) {
        panic!(
            "Argument types differ between wrapper and wrapped function! {}",
            e
        );
    }

    let builder = LLVMCreateBuilderInContext(context);
    LLVMPositionBuilderAtEnd(builder, outer_bb);
    outer_args[0] = LLVMBuildCall(
        builder,
        fnc,
        input_args.as_mut_ptr(),
        input_args.len() as u32,
        c_inner_fnc_name.as_ptr(),
    );
    let _ret = LLVMBuildRetVoid(builder);
    let _terminator = LLVMGetBasicBlockTerminator(outer_bb);
    //assert!(LLVMIsNull(terminator)!=0, "no terminator");
    LLVMDisposeBuilder(builder);

    if let Err(e) = verify_function(outer_fnc) {
        panic!("Creating a wrapper function failed! {}", e);
    }

    outer_fnc
}

/// This function creates and returns a wrapper function 'fnc_name' around the given function.
///
/// The wrapped function is expected to return a struct `{ f64 }` consisting of exactly one `f64` value.
/// The wrapper function will accept the same arguments as the wrapped function and return
/// the inner `f64` instead of the struct.
///
/// # Safety
///
/// The `module`, `context`, and `fnc` must all be valid.
/// The function `fnc` must be part of the given module and return a struct with one f64
/// value and no other content.
/// `u_type` and LLVMTypeOf(fnc) shall only differ by the return type, as specified above.
pub unsafe fn extract_return_type(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    fnc: LLVMValueRef,
    u_type: LLVMTypeRef,
    fnc_name: String,
) -> LLVMValueRef {
    let f_type = LLVMTypeOf(fnc);
    dbg!("Unpacking", fnc_name.clone());
    dbg!("From: ", get_type(f_type), " into ", get_type(u_type));

    let inner_param_num = LLVMCountParams(fnc);
    let (outer_fnc, outer_bb, mut outer_args, inner_args, c_inner_fnc_name) =
        create_wrapper(module, context, fnc, u_type, fnc_name);

    if inner_param_num as usize != outer_args.len() {
        panic!("Args len shouldn't differ. Please report this.");
    }

    if let Err(e) = compare_param_types(outer_args.clone(), inner_args) {
        panic!(
            "Argument types differ between wrapper and wrapped function! {}",
            e
        );
    }

    let builder = LLVMCreateBuilderInContext(context);
    LLVMPositionBuilderAtEnd(builder, outer_bb);
    let struct_ret = LLVMBuildCall(
        builder,
        fnc,
        outer_args.as_mut_ptr(),
        outer_args.len() as u32,
        c_inner_fnc_name.as_ptr(),
    );
    // We can use an arbitrary name here, since it will be used to store a tmp value.
    let inner_grad_name = "foo".to_string();
    let c_inner_grad_name = CString::new(inner_grad_name).unwrap();
    let struct_ret = LLVMBuildExtractValue(builder, struct_ret, 0, c_inner_grad_name.as_ptr());
    let _ret = LLVMBuildRet(builder, struct_ret);
    let _terminator = LLVMGetBasicBlockTerminator(outer_bb);
    //assert!(LLVMIsNull(terminator)!=0, "no terminator");
    LLVMDisposeBuilder(builder);

    if let Err(e) = verify_function(outer_fnc) {
        panic!("Creating a wrapper function failed! {}", e);
    }

    outer_fnc
}

unsafe fn create_wrapper(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    fnc: LLVMValueRef,
    u_type: LLVMTypeRef,
    fnc_name: String,
) -> (
    LLVMValueRef,
    LLVMBasicBlockRef,
    Vec<LLVMValueRef>,
    Vec<LLVMValueRef>,
    CString,
) {
    let inner_fnc_name = "inner_".to_string() + &fnc_name;
    let c_inner_fnc_name = CString::new(inner_fnc_name.clone()).unwrap();
    LLVMSetValueName2(
        fnc,
        c_inner_fnc_name.as_ptr(),
        inner_fnc_name.len() as usize,
    );

    let c_outer_fnc_name = CString::new(fnc_name).unwrap();
    let outer_fnc: LLVMValueRef = LLVMAddFunction(
        module,
        c_outer_fnc_name.as_ptr(),
        LLVMGetElementType(u_type) as LLVMTypeRef,
    );

    let entry = "fnc_entry".to_string();
    let c_entry = CString::new(entry).unwrap();
    let basic_block = LLVMAppendBasicBlockInContext(context, outer_fnc, c_entry.as_ptr());

    let outer_params: Vec<LLVMValueRef> = get_params(outer_fnc);
    let inner_params: Vec<LLVMValueRef> = get_params(fnc);

    (
        outer_fnc,
        basic_block,
        outer_params,
        inner_params,
        c_inner_fnc_name,
    )
}

fn get_params(fnc: LLVMValueRef) -> Vec<LLVMValueRef> {
    unsafe {
        let param_num = LLVMCountParams(fnc) as usize;
        let mut fnc_args: Vec<LLVMValueRef> = vec![];
        fnc_args.reserve(param_num);
        LLVMGetParams(fnc, fnc_args.as_mut_ptr());
        fnc_args.set_len(param_num);
        fnc_args
    }
}
