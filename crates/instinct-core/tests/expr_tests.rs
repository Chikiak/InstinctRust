//! Unit tests for the `expr` module.

use instinct_core::diagnostic::Diagnostic;
use instinct_core::expr::{parse, FakeContext};
use instinct_core::value::Value;

fn eval(text: &str, ctx: &FakeContext) -> Result<Value, String> {
    let expr = parse(text, 1, "test.ins").unwrap();
    expr.eval(ctx)
}

fn eval_diagnostic(text: &str) -> Diagnostic {
    match parse(text, 1, "test.ins") {
        Err(d) => d,
        Ok(_) => panic!("expected a parse error"),
    }
}

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
    assert_eq!(eval("not \"x\"", &ctx).unwrap(), Value::Int(0));

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

#[test]
fn test_expr_syntax_errors_report_line() {
    // unmatched closing paren
    let d = eval_diagnostic("(2 + 3");
    assert_eq!(d.line, 1);
    assert!(d.message.contains("expected"));
    // missing operand
    let d = eval_diagnostic("2 +");
    assert_eq!(d.line, 1);
    // unterminated text
    let d = eval_diagnostic(r#"2 + "unterminated"#);
    assert_eq!(d.line, 1);
    assert!(d.message.contains("unterminated"));
    // unknown character
    let d = eval_diagnostic("2 @ 3");
    assert_eq!(d.line, 1);
    assert!(d.message.contains('@'));
    // integer out of range
    let d = eval_diagnostic("99999999999999999999999");
    assert_eq!(d.line, 1);
    assert!(d.message.contains("out of range"));
    // function call is a compile error
    let d = eval_diagnostic("foo(1, 2)");
    assert_eq!(d.line, 1);
    assert!(d.message.contains("unknown function"));
}

#[test]
fn test_expr_names_resolve_through_context() {
    let ctx = FakeContext::new()
        .with("x", Value::Int(10))
        .with("greeting", Value::Text("hi".to_string()));
    assert_eq!(eval("x", &ctx).unwrap(), Value::Int(10));
    assert_eq!(
        eval("greeting", &ctx).unwrap(),
        Value::Text("hi".to_string())
    );
    // unknown name is a runtime error naming the variable
    let err = eval("undefined_var", &ctx).unwrap_err();
    assert!(err.contains("unknown variable"));
    assert!(err.contains("undefined_var"));
}
