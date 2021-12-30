use crate::FncInfo;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

unsafe fn verify_single(info: &FncInfo, fnc_type: LLVMTypeRef) -> Result<LLVMTypeRef, String> {
    let return_type = LLVMGetReturnType(LLVMGetElementType(fnc_type));

    let num_parameters = LLVMCountParamTypes(fnc_type);

    let mut parameter_types: Vec<LLVMTypeRef> = vec![];
    parameter_types.reserve(num_parameters as usize);
    LLVMGetParamTypes(fnc_type, parameter_types.as_mut_ptr());

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

    // 2. Check that we have one entry in input_activity for each parameter in fnc_type.params
    if num_parameters != info.params.input_activity.len() as u32 {
        let error_msg = format!("Your function has {} parameters, but you gave {} input activity values. Please provide exactly one per parameter!",
                                num_parameters, info.params.input_activity.len());
        return Err(error_msg);
    }

    // 4. (optional) check for LLVMFloatType in params.

    // Now start generating output type

    //
    unreachable!()
}

pub fn verify(
    infos: Vec<FncInfo>,
    primary_functions: Vec<LLVMValueRef>,
) -> Result<Vec<LLVMTypeRef>, String> {
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
    if grad_names.len() != unique_grad_names.len() {
        let error_msg =
            "Please only use unique names for your functions. Double-check your build.rs file.";
        return Err(error_msg.to_string());
    }

    let mut grad_type_refs: Vec<LLVMTypeRef> = vec![];
    for (info, &fnc) in infos.iter().zip(primary_functions.iter()) {
        unsafe {
            let fnc_type = LLVMTypeOf(fnc);
            match verify_single(info, fnc_type) {
                Ok(t) => grad_type_refs.push(t),
                Err(e) => {
                    return Err(format!("Error in function {}: {}", info.primary_name, e));
                }
            }
        }
    }
    Ok(grad_type_refs)
}
