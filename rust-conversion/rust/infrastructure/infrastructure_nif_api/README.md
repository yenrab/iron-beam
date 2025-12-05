# Infrastructure NIF API

This crate will provide the Rust NIF API - equivalent to the C `erl_nif.h` API but implemented in pure Rust.

## Purpose

NIFs (Native Implemented Functions) need to create and decode Erlang terms in memory. The existing `infrastructure_data_handling` crate provides EI format encoding/decoding (for serialization), but NIFs need functions that work with in-memory Erlang terms (Eterm/u64 values).

## Required Functions

### Term Creation (enif_make_*)

- `enif_make_atom(env, name) -> u64` - Create an atom term
- `enif_make_integer(env, value) -> u64` - Create an integer term
- `enif_make_ulong(env, value) -> u64` - Create an unsigned long term
- `enif_make_binary(env, data) -> u64` - Create a binary term
- `enif_make_string(env, string, encoding) -> u64` - Create a string term
- `enif_make_tuple(env, arity, elements) -> u64` - Create a tuple term
- `enif_make_list(env, elements) -> u64` - Create a list term
- `enif_make_list_cell(env, head, tail) -> u64` - Create a cons cell
- `enif_make_map(env, pairs) -> u64` - Create a map term

### Term Decoding (enif_get_*)

- `enif_get_atom(env, term, name_out) -> bool` - Decode an atom term
- `enif_get_int(env, term, value_out) -> bool` - Decode an integer term
- `enif_get_ulong(env, term, value_out) -> bool` - Decode an unsigned long term
- `enif_get_binary(env, term, data_out) -> bool` - Decode a binary term
- `enif_get_string(env, term, string_out) -> bool` - Decode a string term
- `enif_get_tuple(env, term, arity_out, elements_out) -> bool` - Decode a tuple term
- `enif_get_list(env, term, elements_out) -> bool` - Decode a list term
- `enif_get_map(env, term, pairs_out) -> bool` - Decode a map term

### Error Handling

- `enif_make_badarg(env) -> u64` - Create a badarg exception
- `enif_make_badarg_atom(env) -> u64` - Create a badarg atom
- `enif_is_exception(env, term) -> bool` - Check if term is an exception

### Resource Management

- `enif_alloc_resource(type, size) -> *mut c_void` - Allocate a resource
- `enif_release_resource(resource) -> ()` - Release a resource
- `enif_make_resource(env, resource) -> u64` - Create a resource term

## Implementation Notes

1. **Term Representation**: Erlang terms are represented as `u64` values (Eterm)
2. **Term Tagging**: Terms use tagged pointers - lower bits indicate the type
3. **NIF Environment**: Functions take a `*mut c_void` env parameter (NIF environment)
4. **Memory Management**: Terms are managed by the Erlang VM heap
5. **Thread Safety**: NIF functions must be thread-safe

## Dependencies

- `entities_data_handling` - For term type definitions
- `entities_utilities` - For BigNumber, BigRational types
- `infrastructure_data_handling` - For reference implementations (EI format)

## Status

**TODO**: This crate needs to be created. It should provide a complete Rust implementation of the NIF API, replacing the need for C `erl_nif.h`.

