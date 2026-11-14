//! Extracts an i64 argument by index, naming the function on error.
//! Text arguments are rejected; a missing argument reports the function's
//! real arity instead of a hardcoded value.

use crate::value::Value;

/// Extracts the i64 argument at `index`, naming the function and argument on
/// error. Text arguments are rejected. `arity` is the function's declared
/// arity, reported when the argument list is too short.
pub(super) fn arg_as_int(args: &[Value], index: usize, func: &str, arity: usize) -> Result<i64, String> {
    match args.get(index) {
        Some(Value::Int(n)) => Ok(*n),
        Some(Value::Text(_)) => Err(format!(
            "argument {index} of '{func}' must be an integer, got text"
        )),
        None => Err(format!(
            "function '{func}' expects {arity} arguments, got {}",
            args.len()
        )),
    }
}
