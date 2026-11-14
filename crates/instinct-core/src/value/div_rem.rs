//! Integer division and remainder with floor semantics.

use crate::value::Value;

impl Value {
    /// Evaluates integer division (`/`) with floor semantics.
    pub(crate) fn div(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err("division by zero".to_string());
                }
                if *a == i64::MIN && *b == -1 {
                    return Err("integer overflow in division".to_string());
                }
                Ok(Value::Int(floor_div(*a, *b)?))
            }
            (Value::Text(_), _) | (_, Value::Text(_)) => {
                Err("cannot divide non-integer values".to_string())
            }
        }
    }

    /// Evaluates integer remainder (`%`) with floor semantics.
    ///
    /// The remainder takes the sign of the divisor. Uses wrapping arithmetic
    /// for the intermediate product `q * b` so that valid cases such as
    /// `i64::MIN % 3` do not panic spuriously on an intermediate overflow
    /// even though the final remainder fits in `i64`.
    pub(crate) fn rem(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err("modulo by zero".to_string());
                }
                if *a == i64::MIN && *b == -1 {
                    return Err("integer overflow in modulo".to_string());
                }
                let q = floor_div(*a, *b)?;
                let product = q.wrapping_mul(*b);
                let r = a.wrapping_sub(product);
                Ok(Value::Int(r))
            }
            (Value::Text(_), _) | (_, Value::Text(_)) => {
                Err("cannot modulo non-integer values".to_string())
            }
        }
    }
}

/// Floor division: rounds toward minus infinity, matching the enunciado.
///
/// Rust's `/` and `%` truncate toward zero, so we adjust: when there is a
/// remainder and the dividend/divisor signs differ, the quotient is one less
/// than the truncated quotient.
fn floor_div(a: i64, b: i64) -> Result<i64, String> {
    let mut q = a / b;
    let r = a % b;
    if r != 0 && ((r < 0) != (b < 0)) {
        q = q
            .checked_sub(1)
            .ok_or_else(|| "integer overflow in division".to_string())?;
    }
    Ok(q)
}
