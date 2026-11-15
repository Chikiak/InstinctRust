//! Logic, truthiness, short-circuit, and text operation tests.

use super::helpers::*;
use instinct_core::expr::FakeContext;
use instinct_core::value::Value;

#[test]
fn test_expr_logic_truthiness_and_short_circuit() {
    let ctx = FakeContext::new();
    // 0 and empty text are false.
    assert_eq!(eval("0 and 1", &ctx).unwrap(), Value::Int(0));
    assert_eq!(eval("1 and 0", &ctx).unwrap(), Value::Int(0));
    assert_eq!(eval("1 and 1", &ctx).unwrap(), Value::Int(1));
    assert_eq!(eval("0 or 0", &ctx).unwrap(), Value::Int(0));
    assert_eq!(eval("0 or 1", &ctx).unwrap(), Value::Int(1));
    assert_eq!(eval("1 or 0", &ctx).unwrap(), Value::Int(1));
    assert_eq!(eval("not 0", &ctx).unwrap(), Value::Int(1));
    assert_eq!(eval("not 1", &ctx).unwrap(), Value::Int(0));
    assert_eq!(eval("not \"\"", &ctx).unwrap(), Value::Int(1));
    assert_eq!(eval(r#"not "x""#, &ctx).unwrap(), Value::Int(0));

    // Short-circuit: the right side divides by zero but is never reached.
    assert_eq!(eval("0 and 1 / 0", &ctx).unwrap(), Value::Int(0));
    assert_eq!(eval("1 or 1 / 0", &ctx).unwrap(), Value::Int(1));
}

#[test]
fn test_expr_text_operations_and_type_errors() {
    let ctx = FakeContext::new();
    assert_eq!(eval(r#""a" == "a""#, &ctx).unwrap(), Value::Int(1));
    assert_eq!(eval(r#""a" == "b""#, &ctx).unwrap(), Value::Int(0));
    assert_eq!(eval(r#""a" != "b""#, &ctx).unwrap(), Value::Int(1));
    assert_eq!(eval(r#""a" != "a""#, &ctx).unwrap(), Value::Int(0));

    // text with ordering operator is an error
    assert!(eval(r#""a" < "b""#, &ctx).is_err());
    // int with text is an error
    assert!(eval("5 == \"a\"", &ctx).is_err());
    assert!(eval("5 < \"a\"", &ctx).is_err());
    // negate a text is an error
    assert!(eval(r#"-"a""#, &ctx).is_err());
    // add text to text is an error
    assert!(eval(r#""a" + "b""#, &ctx).is_err());
}

#[test]
fn test_expr_hash_inside_string_is_not_a_comment() {
    let ctx = FakeContext::new();
    // The `#` is inside the text, not a comment.
    assert_eq!(
        eval("\"#hello\"", &ctx).unwrap(),
        Value::Text("#hello".to_string())
    );
}
