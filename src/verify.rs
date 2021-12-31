use crate::FncInfo;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

unsafe fn verify_single(info: &FncInfo, fnc_type: LLVMTypeRef) -> Result<(), String> {
    let fnc_type = LLVMGetElementType(fnc_type);
    dbg!(1);
    let return_type = LLVMGetReturnType(fnc_type);

    dbg!(2);
    let num_parameters = LLVMCountParamTypes(fnc_type);

    dbg!(3);
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

    // 4. (optional) check for LLVMFloatType in params.

    Ok(())
}

pub fn verify(infos: Vec<FncInfo>, primary_functions: Vec<LLVMValueRef>) -> Result<(), String> {
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
    if grad_names.len() != unique_grad_names.len() {
        let error_msg =
            "Please only use unique names for your functions. Double-check your build.rs file.";
        return Err(error_msg.to_string());
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
