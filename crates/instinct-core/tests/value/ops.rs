use super::common::{int, text};
use instinct_core::value::Value;

#[test]
fn test_floor_division_and_modulo() {
    assert_eq!(int(7).binary_op("/", &int(2)).unwrap(), int(3));
    assert_eq!(int(-7).binary_op("/", &int(2)).unwrap(), int(-4));
    assert_eq!(int(7).binary_op("/", &int(-2)).unwrap(), int(-4));
    assert_eq!(int(-7).binary_op("%", &int(3)).unwrap(), int(2));
    assert_eq!(int(7).binary_op("%", &int(-3)).unwrap(), int(-2));
}

#[test]
fn test_division_by_zero_and_overflow() {
    assert!(int(7).binary_op("/", &int(0)).is_err());
    assert!(int(7).binary_op("%", &int(0)).is_err());
    assert!(Value::Int(i64::MIN).unary_op("-").is_err());
    assert!(Value::Int(i64::MIN).binary_op("/", &int(-1)).is_err());
    assert!(Value::Int(i64::MIN).binary_op("%", &int(-1)).is_err());
    assert!(Value::Int(i64::MAX).binary_op("+", &int(1)).is_err());
    assert!(Value::Int(i64::MIN).binary_op("-", &int(1)).is_err());
    assert!(Value::Int(i64::MAX).binary_op("*", &int(2)).is_err());
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
    assert_eq!(text("a").binary_op("==", &text("a")).unwrap(), int(1));
    assert_eq!(text("a").binary_op("==", &text("b")).unwrap(), int(0));
    assert_eq!(text("a").binary_op("!=", &text("b")).unwrap(), int(1));
    assert_eq!(text("a").binary_op("!=", &text("a")).unwrap(), int(0));
}

#[test]
fn test_not_any_type() {
    assert_eq!(text("").unary_op("not").unwrap(), int(1));
    assert_eq!(text("a").unary_op("not").unwrap(), int(0));
    assert_eq!(int(0).unary_op("not").unwrap(), int(1));
    assert_eq!(int(5).unary_op("not").unwrap(), int(0));
}

#[test]
fn test_to_bool_int() {
    assert_eq!(int(0).to_bool_int(), 0);
    assert_eq!(int(1).to_bool_int(), 1);
    assert_eq!(int(-5).to_bool_int(), 1);
    assert_eq!(text("").to_bool_int(), 0);
    assert_eq!(text("cualquier texto").to_bool_int(), 1);
}

#[test]
fn test_integer_order_comparisons() {
    assert_eq!(int(1).binary_op("<", &int(2)).unwrap(), int(1));
    assert_eq!(int(2).binary_op("<", &int(1)).unwrap(), int(0));
    assert_eq!(int(1).binary_op("<=", &int(1)).unwrap(), int(1));
    assert_eq!(int(2).binary_op("<=", &int(1)).unwrap(), int(0));
    assert_eq!(int(2).binary_op(">", &int(1)).unwrap(), int(1));
    assert_eq!(int(1).binary_op(">", &int(2)).unwrap(), int(0));
    assert_eq!(int(1).binary_op(">=", &int(1)).unwrap(), int(1));
    assert_eq!(int(0).binary_op(">=", &int(1)).unwrap(), int(0));
    // order with negatives and zero (Python semantics, no floats)
    assert_eq!(int(-3).binary_op("<", &int(-1)).unwrap(), int(1));
    assert_eq!(int(-1).binary_op(">", &int(-3)).unwrap(), int(1));
    assert_eq!(int(0).binary_op(">=", &int(0)).unwrap(), int(1));
}
