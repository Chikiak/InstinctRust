//! Unit tests for the `expr` module.

use instinct_core::diagnostic::Diagnostic;
use instinct_core::expr::{parse, FakeContext};
use instinct_core::functions::FunctionCatalog;
use instinct_core::interp::InterpreterVisitor;
use instinct_core::value::Value;

fn eval(text: &str, ctx: &FakeContext) -> Result<Value, String> {
    let expr = parse(text, 1, "test.ins", &FunctionCatalog::default()).unwrap();
    expr.accept(&mut InterpreterVisitor { ctx })
}

fn eval_diagnostic(text: &str) -> Diagnostic {
    match parse(text, 1, "test.ins", &FunctionCatalog::default()) {
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

// ---------------------------------------------------------------------------
// AC6: Call node — arguments are arbitrary expressions, function resolved by catalog
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// AC4: visitante no-intérprete de demostración (NodeCounter)
// ---------------------------------------------------------------------------

/// Visitor que cuenta nodos del AST. La salida real vive en `self.count`;
/// cada `visit_*` devuelve `Ok(Value::Int(0))` y los compuestos recorren sus
/// hijos con `accept`.
struct NodeCounter {
    count: usize,
}

impl instinct_core::expr::ExprVisitor for NodeCounter {
    type Output = Result<Value, String>;

    fn visit_literal_int(&mut self, _node: &instinct_core::expr::LiteralInt) -> Self::Output {
        self.count += 1;
        Ok(Value::Int(0))
    }

    fn visit_literal_text(&mut self, _node: &instinct_core::expr::LiteralText) -> Self::Output {
        self.count += 1;
        Ok(Value::Int(0))
    }

    fn visit_name(&mut self, _node: &instinct_core::expr::Name) -> Self::Output {
        self.count += 1;
        Ok(Value::Int(0))
    }

    fn visit_call(&mut self, node: &instinct_core::expr::Call) -> Self::Output {
        self.count += 1;
        for arg in &node.args {
            arg.accept(self)?;
        }
        Ok(Value::Int(0))
    }

    fn visit_unary(&mut self, node: &instinct_core::expr::Unary) -> Self::Output {
        self.count += 1;
        node.operand.accept(self)?;
        Ok(Value::Int(0))
    }

    fn visit_binary(&mut self, node: &instinct_core::expr::Binary) -> Self::Output {
        self.count += 1;
        node.left.accept(self)?;
        node.right.accept(self)?;
        Ok(Value::Int(0))
    }
}

#[test]
fn test_node_counter_counts_all_nodes() {
    // `"1 + 2 * 3"` -> Binary(+) { LiteralInt(1), Binary(*) { LiteralInt(2), LiteralInt(3) } }
    let expr = parse("1 + 2 * 3", 1, "test.ins", &FunctionCatalog::default()).unwrap();
    let mut counter = NodeCounter { count: 0 };
    expr.accept(&mut counter).unwrap();
    assert_eq!(counter.count, 5, "debe contar los 5 nodos del AST");
}
