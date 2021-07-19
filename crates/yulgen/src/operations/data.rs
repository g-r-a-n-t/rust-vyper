use crate::operations::abi as abi_operations;
use crate::types::{to_abi_types, EvmSized, to_abi_selector_names};
use fe_analyzer::namespace::events::EventDef;
use fe_analyzer::namespace::types::{Array, FixedSize};
use yultsur::*;

/// Loads a value of the given type from storage.
pub fn sload<T: EvmSized>(typ: T, sptr: yul::Expression) -> yul::Expression {
    let size = literal_expression! { (typ.size()) };
    expression! { bytes_sloadn([sptr], [size]) }
}

/// Stores a value of the given type in storage.
pub fn sstore<T: EvmSized>(
    typ: T,
    sptr: yul::Expression,
    value: yul::Expression,
) -> yul::Statement {
    let size = literal_expression! { (typ.size()) };
    statement! { bytes_sstoren([sptr], [size], [value]) }
}

/// Loads a value of the given type from memory.
pub fn mload<T: EvmSized>(typ: T, mptr: yul::Expression) -> yul::Expression {
    let size = literal_expression! { (typ.size()) };
    expression! { mloadn([mptr], [size]) }
}

/// Stores a value of the given type in memory.
pub fn mstore<T: EvmSized>(
    typ: T,
    mptr: yul::Expression,
    value: yul::Expression,
) -> yul::Statement {
    let size = literal_expression! { (typ.size()) };
    statement! { mstoren([mptr], [size], [value]) }
}

/// Copies a segment of memory into storage.
pub fn mcopys<T: EvmSized>(typ: T, sptr: yul::Expression, mptr: yul::Expression) -> yul::Statement {
    let size = literal_expression! { (typ.size()) };
    let word_ptr = expression! { div([sptr], 32) };
    statement! { mcopys([mptr], [word_ptr], [size]) }
}

/// Copies a segment of storage into memory.
///
/// Returns the address of the data in memory.
pub fn scopym<T: EvmSized>(typ: T, sptr: yul::Expression) -> yul::Expression {
    let size = literal_expression! { (typ.size()) };
    let word_ptr = expression! { div([sptr], 32) };
    expression! { scopym([word_ptr], [size]) }
}

/// Copies a segment of storage to another segment of storage.
pub fn scopys<T: EvmSized>(
    typ: T,
    dest_ptr: yul::Expression,
    origin_ptr: yul::Expression,
) -> yul::Statement {
    let size = literal_expression! { (typ.size()) };
    let origin_word = expression! { div([origin_ptr], 32) };
    let dest_word = expression! { div([dest_ptr], 32) };
    statement! { scopys([origin_word], [dest_word], [size]) }
}

/// Copies a segment of memory to another segment of memory.
pub fn mcopym<T: EvmSized>(typ: T, ptr: yul::Expression) -> yul::Expression {
    let size = literal_expression! { (typ.size()) };
    expression! { mcopym([ptr], [size]) }
}

/// Logs an event.
pub fn emit_event(event: EventDef, vals: Vec<yul::Expression>) -> yul::Statement {
    let topic_0 = fe_abi::utils::event_topic(
        &event.name,
        &to_abi_selector_names(&event.all_field_types())
    );
    let mut topics = vec![literal_expression! { (topic_0) }];

    let (field_vals, field_types): (Vec<yul::Expression>, Vec<FixedSize>) = event
        .non_indexed_field_types_with_index()
        .into_iter()
        .map(|(index, typ)| (vals[index].to_owned(), typ))
        .unzip();

    // field types will be relevant when we implement indexed array values
    let (mut indexed_field_vals, _): (Vec<yul::Expression>, Vec<FixedSize>) = event
        .indexed_field_types_with_index()
        .into_iter()
        .map(|(index, typ)| (vals[index].to_owned(), typ))
        .unzip();

    let encoding = abi_operations::encode(&to_abi_types(&field_types), field_vals);
    let encoding_size = abi_operations::encoding_size(&to_abi_types(&field_types), vals);

    // for now we assume these are all base type values and therefore do not need to
    // be hashed
    topics.append(&mut indexed_field_vals);

    let log_func = identifier! { (format!("log{}", topics.len())) };

    return statement! { [log_func]([encoding], [encoding_size], [topics...]) };
}

/// Sums a list of expressions using nested add operations.
pub fn sum(vals: Vec<yul::Expression>) -> yul::Expression {
    if vals.is_empty() {
        return expression! { 0 };
    }

    vals.into_iter()
        .reduce(|val1, val2| expression! { add([val1], [val2]) })
        .unwrap()
}

/// Hashes the storage nonce of a map with a key to determine the value's
/// location in storage.
pub fn keyed_map(map: yul::Expression, key: yul::Expression) -> yul::Expression {
    expression! { map_value_ptr([map], [key]) }
}

/// Finds the location of an array element base on the element size, element
/// index, and array location.
pub fn indexed_array(
    typ: Array,
    array: yul::Expression,
    index: yul::Expression,
) -> yul::Expression {
    let inner_size = literal_expression! { (typ.inner.size()) };
    expression! { add([array], (mul([index], [inner_size]))) }
}
