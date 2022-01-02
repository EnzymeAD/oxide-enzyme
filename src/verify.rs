use crate::{get_type, FncInfo};
use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule};
use llvm_sys::core::*;
use llvm_sys::prelude::*;
use std::ffi::CStr;
use std::ptr;

unsafe fn verify_single(info: &FncInfo, fnc_type: LLVMTypeRef) -> Result<(), String> {
    let fnc_type = LLVMGetElementType(fnc_type);
    let return_type = LLVMGetReturnType(fnc_type);

    let num_parameters = LLVMCountParamTypes(fnc_type);

    let mut parameter_types: Vec<LLVMTypeRef> = vec![];
    parameter_types.reserve(num_parameters as usize);
    LLVMGetParamTypes(fnc_type, parameter_types.as_mut_ptr());

    dbg!("First local check");
    // 1. Check that info.ret_info == None if fnc_type returns void
    if return_type == LLVMVoidType() {
        if info.params.ret_info.is_some() {
            let error_msg = "Your function is returning (), so please set the ret_info of your FncInfo to None!".to_string();
            return Err(error_msg);
        }
    } else if info.params.ret_info.is_none() {
        let error_msg = "Your function is returning something, so please don't set the ret_info of your FncInfo to None!".to_string();
        return Err(error_msg);
    }

    dbg!("Second local check");
    // 2. Check that we have one entry in input_activity for each parameter in fnc_type.params
    if num_parameters != info.params.input_activity.len() as u32 {
        let error_msg = format!("Your function has {} parameters, but you gave {} input activity values. Please provide exactly one per parameter!",
                                num_parameters, info.params.input_activity.len());
        return Err(error_msg);
    }

    // 3. (optional) check for LLVMFloatType in params.

    Ok(())
}

pub fn verify_user_inputs(
    infos: Vec<FncInfo>,
    primary_functions: Vec<LLVMValueRef>,
) -> Result<(), String> {
    dbg!("First global check");
    if infos.len() != primary_functions.len() {
        let error_msg = format!(
            "Number of primary functions and function informations differ. \
            This should have been caught earlier, please report it! {} {}",
            infos.len(),
            primary_functions.len()
        );
        return Err(error_msg);
    }

    let mut grad_names = vec![];
    for info in &infos {
        grad_names.push(info.grad_name.clone());
    }
    let mut unique_grad_names = grad_names.clone();
    unique_grad_names.sort();
    unique_grad_names.dedup();
    dbg!("Second global check");
    for i in 0..(unique_grad_names.len() - 1) {
        if unique_grad_names[i] == unique_grad_names[i + 1] {
            let error_msg = format!(
                "You are assigning multiple gradient functions to {}. \
                             Please double-check your build.rs file.",
                unique_grad_names[i]
            );
            return Err(error_msg);
        }
    }

    dbg!("Moving to local checks");
    for (info, &fnc) in infos.iter().zip(primary_functions.iter()) {
        unsafe {
            let fnc_type = LLVMTypeOf(fnc);
            verify_single(info, fnc_type)?;
        }
    }
    Ok(())
}

pub unsafe fn verify_function(fnc: LLVMValueRef) -> Result<(), String> {
    let fnc_ok = LLVMVerifyFunction(fnc, LLVMVerifierFailureAction::LLVMAbortProcessAction) == 0;
    if fnc_ok {
        Ok(())
    } else {
        Err("Could not validate function!".to_string())
    }
}

pub unsafe fn verify_module(module: LLVMModuleRef) -> Result<(), String> {
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

pub fn compare_param_types(
    args1: Vec<LLVMValueRef>,
    args2: Vec<LLVMValueRef>,
) -> Result<(), String> {
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
