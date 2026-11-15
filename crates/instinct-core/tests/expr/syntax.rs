//! Syntax error diagnostic tests.

use super::helpers::*;
use instinct_core::expr::FakeContext;
use instinct_core::value::Value;

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
    // unknown function is a compile error
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
