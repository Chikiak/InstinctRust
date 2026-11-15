//! Operator precedence tests.

use super::helpers::*;
use instinct_core::expr::FakeContext;
use instinct_core::value::Value;

#[test]
fn test_expr_precedence_table() {
    let ctx = FakeContext::new();
    assert_eq!(eval("2 + 3 * 4", &ctx).unwrap(), Value::Int(14));
    assert_eq!(eval("(2 + 3) * 4", &ctx).unwrap(), Value::Int(20));
    assert_eq!(eval("-2 * 3", &ctx).unwrap(), Value::Int(-6));
    assert_eq!(eval("10 - 4 - 3", &ctx).unwrap(), Value::Int(3));
    // `not` binds tighter than `==`: (not 0) == 5 => 1 == 5 => 0
    assert_eq!(eval("not 0 == 5", &ctx).unwrap(), Value::Int(0));
    // `and` binds tighter than `or`: 1 or (0 and 0) => 1 or 0 => 1
    assert_eq!(eval("1 or 0 and 0", &ctx).unwrap(), Value::Int(1));
}

#[test]
fn test_expr_floor_division_and_modulo() {
    let ctx = FakeContext::new();
    assert_eq!(eval("7 / 2", &ctx).unwrap(), Value::Int(3));
    assert_eq!(eval("-7 / 2", &ctx).unwrap(), Value::Int(-4));
    assert_eq!(eval("7 / -2", &ctx).unwrap(), Value::Int(-4));
    assert_eq!(eval("-7 % 3", &ctx).unwrap(), Value::Int(2));
    assert_eq!(eval("7 % -3", &ctx).unwrap(), Value::Int(-2));
}

#[test]
fn test_expr_division_by_zero_and_overflow() {
    let ctx = FakeContext::new();
    let err = eval("7 / 0", &ctx).unwrap_err();
    assert_eq!(err, "division by zero");
    let err = eval("7 % 0", &ctx).unwrap_err();
    assert_eq!(err, "modulo by zero");
    let err = eval("9223372036854775807 + 1", &ctx).unwrap_err();
    assert!(err.contains("overflow"));
    let err = eval("9223372036854775807 * 2", &ctx).unwrap_err();
    assert!(err.contains("overflow"));
}
