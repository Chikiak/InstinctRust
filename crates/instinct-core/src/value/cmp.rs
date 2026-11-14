//! Comparison operators for [`Value`].

use crate::value::Value;

impl Value {
    pub(crate) fn eq(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(if a == b { 1 } else { 0 })),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Int(if a == b { 1 } else { 0 })),
            _ => Err("cannot compare an integer and a text".to_string()),
        }
    }

    pub(crate) fn ne(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(if a != b { 1 } else { 0 })),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Int(if a != b { 1 } else { 0 })),
            _ => Err("cannot compare an integer and a text".to_string()),
        }
    }

    pub(crate) fn cmp_ord<F>(&self, other: &Value, compare: F) -> Result<Value, String>
    where
        F: Fn(i64, i64) -> bool,
    {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(if compare(*a, *b) { 1 } else { 0 })),
            (Value::Text(_), Value::Text(_)) => {
                Err("cannot compare texts with an ordering operator".to_string())
            }
            _ => Err("cannot compare an integer and a text".to_string()),
        }
    }
}
