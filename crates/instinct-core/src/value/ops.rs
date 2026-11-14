//! Binary and unary operators on [`Value`].

use crate::value::Value;

impl Value {
    /// Evaluates a binary operator between two values.
    ///
    /// Returns a runtime error on type mismatches, division/modulo by zero,
    /// and arithmetic overflow. Text only supports `==` and `!=`.
    pub fn binary_op(&self, op: &str, other: &Value) -> Result<Value, String> {
        match op {
            "+" => self.add(other),
            "-" => self.sub(other),
            "*" => self.mul(other),
            "/" => self.div(other),
            "%" => self.rem(other),
            "<" => self.cmp_ord(other, |a, b| a < b),
            "<=" => self.cmp_ord(other, |a, b| a <= b),
            ">" => self.cmp_ord(other, |a, b| a > b),
            ">=" => self.cmp_ord(other, |a, b| a >= b),
            "==" => self.eq(other),
            "!=" => self.ne(other),
            _ => Err(format!("unknown binary operator '{op}'")),
        }
    }

    /// Evaluates a unary operator (`-` or `not`) on a value.
    pub fn unary_op(&self, op: &str) -> Result<Value, String> {
        match op {
            "-" => self.neg(),
            "not" => Ok(Value::Int(if self.is_truthy() { 0 } else { 1 })),
            _ => Err(format!("unknown unary operator '{op}'")),
        }
    }

    fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_add(*b)
                .map(Value::Int)
                .ok_or_else(|| "integer overflow in addition".to_string()),
            (Value::Text(_), Value::Text(_)) => Err("cannot add text to text".to_string()),
            _ => Err("cannot add an integer and a text".to_string()),
        }
    }

    fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_sub(*b)
                .map(Value::Int)
                .ok_or_else(|| "integer overflow in subtraction".to_string()),
            (Value::Text(_), Value::Text(_)) => Err("cannot subtract text from text".to_string()),
            _ => Err("cannot subtract an integer and a text".to_string()),
        }
    }

    fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a
                .checked_mul(*b)
                .map(Value::Int)
                .ok_or_else(|| "integer overflow in multiplication".to_string()),
            (Value::Text(_), Value::Text(_)) => Err("cannot multiply text by text".to_string()),
            _ => Err("cannot multiply an integer and a text".to_string()),
        }
    }

    fn neg(&self) -> Result<Value, String> {
        match self {
            Value::Int(a) => {
                if *a == i64::MIN {
                    return Err("integer overflow in unary minus".to_string());
                }
                Ok(Value::Int(-a))
            }
            Value::Text(_) => Err("cannot negate a text".to_string()),
        }
    }
}
