use crate::get_type;
use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyModule};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ffi::{CStr, CString};
use std::ptr;

// We probably should add some use of LLVMVerifyFunction, at least while developing
#[allow(unused_imports)]
use llvm_sys::analysis::LLVMVerifyFunction;

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
    let param_num = LLVMCountParamTypes(LLVMGetElementType(f_type));
    let mut param_types: Vec<LLVMTypeRef> = vec![];
    param_types.reserve(param_num as usize);
    LLVMGetParamTypes(LLVMGetElementType(f_type), param_types.as_mut_ptr());
    let inner_fnc_name = "struct_".to_string() + &fnc_name;
    let c_inner_fnc_name = CString::new(inner_fnc_name.clone()).unwrap();
    let outer_fnc_name = fnc_name;
    let c_outer_fnc_name = CString::new(outer_fnc_name).unwrap();
    let new_fnc: LLVMValueRef = LLVMAddFunction(
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
    let basic_block = LLVMAppendBasicBlockInContext(context, new_fnc, c_entry.as_ptr());
    let mut fnc_args: Vec<LLVMValueRef> = vec![];
    fnc_args.reserve(param_num as usize);
    LLVMGetParams(new_fnc, fnc_args.as_mut_ptr());

    let builder = LLVMCreateBuilderInContext(context);
    LLVMPositionBuilderAtEnd(builder, basic_block);
    let struct_ret = LLVMBuildCall(
        builder,
        fnc,
        fnc_args.as_mut_ptr(),
        param_num,
        c_inner_fnc_name.as_ptr(),
    );
    // We can use an arbitrary name here, since it will be an internal wrapper
    let inner_grad_name = "foo".to_string();
    let c_inner_grad_name = CString::new(inner_grad_name).unwrap();
    let struct_ret = LLVMBuildExtractValue(builder, struct_ret, 0, c_inner_grad_name.as_ptr());
    let _ret = LLVMBuildRet(builder, struct_ret);
    let _terminator = LLVMGetBasicBlockTerminator(basic_block);
    //assert!(LLVMIsNull(terminator)!=0, "no terminator");
    LLVMDisposeBuilder(builder);

    assert!(
        LLVMVerifyFunction(new_fnc, LLVMVerifierFailureAction::LLVMAbortProcessAction) == 0,
        "Could not validate function!"
    );

    let mut msg = ptr::null_mut();
    assert!(
        LLVMVerifyModule(
            module,
            LLVMVerifierFailureAction::LLVMReturnStatusAction,
            &mut msg
        ) == 0,
        "Could not validate! {:?}",
        CStr::from_ptr(msg).to_str().unwrap()
    );
    LLVMDisposeMessage(msg);

    new_fnc
}
