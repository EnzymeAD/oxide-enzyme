use crate::get_type;
use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ffi::{CStr, CString};
use std::ptr;

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

    let inner_ret_type = LLVMGetReturnType(LLVMGetElementType(u_type));
    if inner_ret_type != LLVMVoidType() {
        let is = CString::from_raw(LLVMPrintTypeToString(inner_ret_type));
        let should = CString::from_raw(LLVMPrintTypeToString(LLVMVoidType()));
        panic!(
            "Return struct isn't moved into args. Please report this. {} vs. {}",
            is.to_str().unwrap(),
            should.to_str().unwrap()
        );
    }

    let (outer_fnc, outer_bb, mut outer_args, inner_args, c_inner_fnc_name) =
        create_wrapper(module, context, fnc, u_type, fnc_name);
    let inner_param_num = LLVMCountParamTypes(LLVMGetElementType(f_type));
    assert_eq!(
        1 + inner_param_num as usize,
        outer_args.len(),
        "Outer wrapper should have exactly one extra arg. Please report this. {} vs {}",
        inner_param_num,
        outer_args.len()
    );

    let builder = LLVMCreateBuilderInContext(context);
    LLVMPositionBuilderAtEnd(builder, outer_bb);

    let mut input_args = outer_args.split_off(1);
    let out_extra_arg = LLVMTypeOf(outer_args[0]);
    assert_eq!(
        inner_ret_type, out_extra_arg,
        "Ret of inner should be identical to first param of outer. Please report this."
    );
    if let Err(e) = compare_param_types(outer_args.clone(), inner_args) {
        panic!(
            "Argument types differ between wrapper and wrapped function! {}",
            e
        );
    }

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

    if let Err(e) = verify(module, outer_fnc) {
        panic!("Creating a wrapper failed! {}", e);
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

    let (outer_fnc, outer_basic_block, mut outer_args, inner_args, c_inner_fnc_name) =
        create_wrapper(module, context, fnc, u_type, fnc_name);
    let inner_param_num = LLVMCountParamTypes(LLVMGetElementType(f_type));
    assert_eq!(
        inner_param_num as usize,
        outer_args.len(),
        "Args len shouldn't differ. Please report this."
    );
    if let Err(e) = compare_param_types(outer_args.clone(), inner_args) {
        panic!(
            "Argument types differ between wrapper and wrapped function! {}",
            e
        );
    }

    let builder = LLVMCreateBuilderInContext(context);
    LLVMPositionBuilderAtEnd(builder, outer_basic_block);
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
    let _terminator = LLVMGetBasicBlockTerminator(outer_basic_block);
    //assert!(LLVMIsNull(terminator)!=0, "no terminator");
    LLVMDisposeBuilder(builder);

    if let Err(e) = verify(module, outer_fnc) {
        panic!("Creating a wrapper failed! {}", e);
    }

    outer_fnc
}

unsafe fn compare_param_types(
    args1: Vec<LLVMValueRef>,
    args2: Vec<LLVMValueRef>,
) -> Result<(), String> {
    for (i, (a, b)) in args1.iter().zip(args2.iter()).enumerate() {
        if LLVMTypeOf(*a) != LLVMTypeOf(*b) {
            return Err(format!(
                "Type of inputs between wrapper and wrapped fnc differ at {}",
                i
            ));
        }
    }
    Ok(())
}

unsafe fn get_params(fnc: LLVMValueRef) -> Vec<LLVMValueRef> {
    let u_type: LLVMTypeRef = LLVMTypeOf(fnc);
    let param_num = LLVMCountParamTypes(LLVMGetElementType(u_type));
    let mut fnc_args: Vec<LLVMValueRef> = vec![];
    fnc_args.reserve(param_num as usize);
    LLVMGetParams(fnc, fnc_args.as_mut_ptr());
    fnc_args
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
    let c_outer_fnc_name = CString::new(fnc_name).unwrap();
    let outer_fnc: LLVMValueRef = LLVMAddFunction(
        module,
        c_outer_fnc_name.as_ptr(),
        LLVMGetElementType(u_type) as LLVMTypeRef,
    );
    LLVMSetValueName2(
        fnc,
        c_inner_fnc_name.as_ptr(),
        inner_fnc_name.len() as usize,
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

unsafe fn verify(module: LLVMModuleRef, fnc: LLVMValueRef) -> Result<(), String> {
    let fnc_ok = LLVMVerifyFunction(fnc, LLVMVerifierFailureAction::LLVMAbortProcessAction) == 0;
    if !fnc_ok {
        return Err("Could not validate function!".to_string());
    };

    let mut msg = ptr::null_mut();
    let module_ok = LLVMVerifyModule(
        module,
        LLVMVerifierFailureAction::LLVMReturnStatusAction,
        &mut msg,
    ) == 0;
    if !module_ok {
        let c_msg = CStr::from_ptr(msg)
            .to_str()
            .expect("This msg should have been created by llvm!");
        let error_msg = "Could not validate module!".to_owned() + c_msg;
        LLVMDisposeMessage(msg);
        return Err(error_msg);
    }
    Ok(())
}
