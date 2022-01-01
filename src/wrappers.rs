use crate::get_type;
use crate::verify::verify_function;
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ffi::CString;

pub unsafe fn move_return_into_args(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    fnc: LLVMValueRef,
    u_type: LLVMTypeRef,
    f_type: LLVMTypeRef,
    fnc_name: String,
) -> LLVMValueRef {
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

// Our Gradient fnc is returning a struct containing one element.
// Our Rust code expects a function returning the element, without the struct
// We create a new (identical) fnc which only differs in returning T rather than { T }.
// All it does is call enzyme's grad fnc and extract T from the struct, forwarding it.
pub unsafe fn extract_return_type(
    module: LLVMModuleRef,
    context: LLVMContextRef,
    fnc: LLVMValueRef,
    u_type: LLVMTypeRef,
    f_type: LLVMTypeRef,
    fnc_name: String,
) -> LLVMValueRef {
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

fn compare_param_types(args1: Vec<LLVMValueRef>, args2: Vec<LLVMValueRef>) -> Result<(), String> {
    for (i, (a, b)) in args1.iter().zip(args2.iter()).enumerate() {
        let type1 = unsafe { LLVMTypeOf(*a) };
        let type2 = unsafe { LLVMTypeOf(*b) };
        if type1 != type2 {
            let type1 = get_type(type1);
            let type2 = get_type(type2);
            return Err(format!(
                "Type of inputs differ at position {}. {:?} vs. {:?}",
                i, type1, type2
            ));
        }
    }
    Ok(())
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
