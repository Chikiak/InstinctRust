//! Runtime values: 64-bit integers or text, with the evaluation semantics
//! of the Instinct language.
//!
//! Integer division rounds toward minus infinity (floor division) and the
//! remainder takes the sign of the divisor, matching the enunciado. All
//! arithmetic is checked: overflow, division by zero and modulo by zero are
//! runtime errors.

/// A value in the Instinct language: a signed 64-bit integer or text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Text(String),
}

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

    fn div(&self, other: &Value) -> Result<Value, String> {
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

    fn rem(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => {
                if *b == 0 {
                    return Err("modulo by zero".to_string());
                }
                if *a == i64::MIN && *b == -1 {
                    return Err("integer overflow in modulo".to_string());
                }
                let q = floor_div(*a, *b)?;
                let product = q
                    .checked_mul(*b)
                    .ok_or_else(|| "integer overflow in modulo".to_string())?;
                let r = a
                    .checked_sub(product)
                    .ok_or_else(|| "integer overflow in modulo".to_string())?;
                Ok(Value::Int(r))
            }
            (Value::Text(_), _) | (_, Value::Text(_)) => {
                Err("cannot modulo non-integer values".to_string())
            }
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

    fn eq(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(if a == b { 1 } else { 0 })),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Int(if a == b { 1 } else { 0 })),
            _ => Err("cannot compare an integer and a text".to_string()),
        }
    }

    fn ne(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(if a != b { 1 } else { 0 })),
            (Value::Text(a), Value::Text(b)) => Ok(Value::Int(if a != b { 1 } else { 0 })),
            _ => Err("cannot compare an integer and a text".to_string()),
        }
    }

    fn cmp_ord<F>(&self, other: &Value, compare: F) -> Result<Value, String>
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

#[cfg(test)]
mod tests {
    use super::*;

    fn int(n: i64) -> Value {
        Value::Int(n)
    }

    fn text(s: &str) -> Value {
        Value::Text(s.to_string())
    }

    #[test]
    fn test_value_is_truthy() {
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Int(1).is_truthy());
        assert!(Value::Int(-1).is_truthy());
        assert!(!Value::Text("".to_string()).is_truthy());
        assert!(Value::Text("a".to_string()).is_truthy());
    }

    #[test]
    fn test_floor_division_and_modulo() {
        assert_eq!(int(7).div(&int(2)).unwrap(), int(3));
        assert_eq!(int(-7).div(&int(2)).unwrap(), int(-4));
        assert_eq!(int(7).div(&int(-2)).unwrap(), int(-4));
        assert_eq!(int(-7).rem(&int(3)).unwrap(), int(2));
        assert_eq!(int(7).rem(&int(-3)).unwrap(), int(-2));
    }

    #[test]
    fn test_division_by_zero_and_overflow() {
        assert!(int(7).div(&int(0)).is_err());
        assert!(int(7).rem(&int(0)).is_err());
        assert!(Value::Int(i64::MIN).neg().is_err());
        assert!(Value::Int(i64::MIN).div(&int(-1)).is_err());
        assert!(Value::Int(i64::MIN).rem(&int(-1)).is_err());
        assert!(Value::Int(i64::MAX).add(&int(1)).is_err());
        assert!(Value::Int(i64::MIN).sub(&int(1)).is_err());
        assert!(Value::Int(i64::MAX).mul(&int(2)).is_err());
    }

    #[test]
    fn test_value_type_errors() {
        // text with ordering operator
        assert!(text("a").binary_op("<", &text("b")).is_err());
        // int with text
        assert!(int(5).binary_op("==", &text("5")).is_err());
        assert!(int(5).binary_op("<", &text("5")).is_err());
        // negate text
        assert!(text("a").unary_op("-").is_err());
        // add text to text
        assert!(text("a").binary_op("+", &text("b")).is_err());
        // text by text multiply
        assert!(text("a").binary_op("*", &text("b")).is_err());
    }

    #[test]
    fn test_text_eq_ne() {
        assert_eq!(text("a").eq(&text("a")).unwrap(), int(1));
        assert_eq!(text("a").eq(&text("b")).unwrap(), int(0));
        assert_eq!(text("a").ne(&text("b")).unwrap(), int(1));
        assert_eq!(text("a").ne(&text("a")).unwrap(), int(0));
    }

    #[test]
    fn test_not_any_type() {
        assert_eq!(text("").unary_op("not").unwrap(), int(1));
        assert_eq!(text("a").unary_op("not").unwrap(), int(0));
        assert_eq!(int(0).unary_op("not").unwrap(), int(1));
        assert_eq!(int(5).unary_op("not").unwrap(), int(0));
    }
}
