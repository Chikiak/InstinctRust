//! Call expression tests.

use super::helpers::*;
use instinct_core::expr::{parse, FakeContext};
use instinct_core::functions::FunctionCatalog;
use instinct_core::interp::InterpreterVisitor;

#[test]
fn test_call_arguments_are_expressions() {
    // `see(1 + 0, 2)` — argument is a binary expression, resolves to see(1, 2)
    let ctx = FakeContext::new();
    let cat = FunctionCatalog::default();
    let expr = parse("see(1 + 0, 2)", 1, "test.ins", &cat).unwrap();
    // FakeContext has no view, so call should fail at runtime with "no creature view"
    let err = expr
        .accept(&mut InterpreterVisitor { ctx: &ctx })
        .unwrap_err();
    assert!(err.contains("no creature view"), "error: {err}");

    // Wrong arity: see with 1 argument
    let d = eval_diagnostic("see(1)");
    assert_eq!(d.line, 1);
    assert!(d.message.contains("see"));
    assert!(d.message.contains("arguments"));

    // Wrong arity: see with 3 arguments
    let d = eval_diagnostic("see(1, 2, 3)");
    assert_eq!(d.line, 1);
    assert!(d.message.contains("see"));
    assert!(d.message.contains("arguments"));

    // No arguments: see()
    let d = eval_diagnostic("see()");
    assert_eq!(d.line, 1);
    assert!(d.message.contains("see"));
    assert!(d.message.contains("arguments"));
}

#[test]
fn test_function_in_expression_reports_on_its_line() {
    // `see` is now a valid function, so use an unknown function name.
    let d = eval_diagnostic("bogus(1, 2)");
    assert_eq!(d.line, 1);
    assert!(d.message.contains("unknown function"));
    assert!(d.message.contains("bogus"));
}
