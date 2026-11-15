//! Shared helpers for expression tests.

use instinct_core::diagnostic::Diagnostic;
use instinct_core::expr::{parse, FakeContext};
use instinct_core::functions::FunctionCatalog;
use instinct_core::interp::InterpreterVisitor;
use instinct_core::value::Value;

/// Parses `text` and evaluates it with `ctx`, returning the interpreter result.
pub fn eval(text: &str, ctx: &FakeContext) -> Result<Value, String> {
    let expr = parse(text, 1, "test.ins", &FunctionCatalog::default()).unwrap();
    expr.accept(&mut InterpreterVisitor { ctx })
}

/// Parses `text` and returns the diagnostic when it fails to parse.
pub fn eval_diagnostic(text: &str) -> Diagnostic {
    match parse(text, 1, "test.ins", &FunctionCatalog::default()) {
        Err(d) => d,
        Ok(_) => panic!("expected a parse error"),
    }
}
