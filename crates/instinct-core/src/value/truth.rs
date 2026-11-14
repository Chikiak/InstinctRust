//! Truthiness and boolean conversion for [`Value`].

use crate::value::Value;

impl Value {
    /// Returns `true` for a non-zero integer or a non-empty text.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Int(n) => *n != 0,
            Value::Text(s) => !s.is_empty(),
        }
    }

    /// Returns `1` if the value is truthy, `0` otherwise.
    pub fn to_bool_int(&self) -> i64 {
        if self.is_truthy() {
            1
        } else {
            0
        }
    }
}
