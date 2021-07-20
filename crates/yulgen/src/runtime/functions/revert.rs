use crate::names;
use crate::names::abi as abi_names;
use crate::operations::abi as abi_operations;
use crate::types::{to_abi_selector_names, to_abi_types, AbiType};
use fe_abi::utils as abi_utils;
use fe_analyzer::namespace::types::{FixedSize, Struct, U256};
use yultsur::*;

/// Generate a YUL function to revert with the `Error` signature and the
/// given set of params.
/// NOTE: This is currently used for `assert False, "message"` statements which are
/// encoded as `Error(msg="message")`. This will be removed in the future.
pub fn generate_revert_fn_for_assert(params: &[AbiType]) -> yul::Statement {
    generate_revert_fn("Error", params)
}

/// Generate a YUL function to revert with a specific struct used as error data
pub fn generate_struct_revert(val: &Struct) -> yul::Statement {
    let typ = AbiType::from(val);
    generate_revert_fn(&val.name, &[typ])
}

/// Generate a YUL function to revert with panic codes
pub fn generate_revert_fn_for_panic() -> yul::Statement {
    let selector = fe_abi::utils::func_selector("Panic", &["uint256".to_string()]);
    let selector = literal_expression! { (selector) };

    function_definition! {
        function revert_with_panic(error_code) {
            (let ptr := alloc_mstoren([selector], 4))
            (pop((alloc_mstoren(error_code, 32))))
            (revert(ptr, (add(4, 32))))
        }
    }
}

/// Generate a YUL function to revert with data
pub fn generate_revert_fn(name: &str, params: &[AbiType]) -> yul::Statement {
    let function_name = names::revert_name(name, params);
    let (param_idents, param_exprs) = abi_names::vals("params", params.len());
    let selector = fe_abi::utils::func_selector(name, &to_abi_selector_names(params));
    let selector = literal_expression! { (selector) };
    let encode_params = abi_operations::encode(params, param_exprs.clone());
    let encoding_size = abi_operations::encoding_size(params, param_exprs);

    function_definition! {
        function [function_name]([param_idents...]) {
            (let ptr := alloc_mstoren([selector], 4))
            (pop([encode_params]))
            (revert(ptr, (add(4, [encoding_size]))))
        }
    }
}

/// Return all revert runtime functions
pub fn all() -> Vec<yul::Statement> {
    vec![generate_revert_fn_for_panic()]
}
