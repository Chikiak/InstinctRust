//! Tests that `Line::clone` preserves expression, arguments and source
//! line for every variant.

use super::compile_text;
use instinct_core::expr::{Expr, FakeContext};
use instinct_core::interp::InterpreterVisitor;
use instinct_core::program::{Line, Program};
use instinct_core::value::Value;

/// Evaluates `expr` with `health = value` and returns whether the result
/// is Int(1).
fn eval_expr(expr: &dyn Expr, value: i64) -> bool {
    let ctx = FakeContext::new().with("health", Value::Int(value));
    matches!(
        expr.accept(&mut InterpreterVisitor { ctx: &ctx }),
        Ok(Value::Int(1))
    )
}

/// Finds the first line satisfying `f` (the program is borrowed).
fn find_line(program: &Program, f: impl Fn(&Line) -> bool) -> &Line {
    program
        .lines
        .iter()
        .find(|l| f(l))
        .expect("expected a program line in the test")
}

#[test]
fn test_line_clone_all_variants() {
    // `line` stores the source line number, not the index in `lines`.
    let text = "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\ngoto loop\nif health > 0 goto loop\nenergy = 5\nwait(1)\nloop:\n";
    let result = compile_text(text);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );
    let program = result.program;

    // ---- Label { name, line }: the clone preserves name and source line ----
    let start = find_line(
        &program,
        |l| matches!(l, Line::Label { name, .. } if name == "start"),
    );
    let (orig_name, orig_line) = match start {
        Line::Label { name, line } => (name.clone(), *line),
        _ => panic!("expected Label"),
    };
    match start.clone() {
        Line::Label { name, line } => {
            assert_eq!(name, "start");
            assert_eq!(name, orig_name);
            assert_eq!(line, orig_line);
        }
        _ => panic!("expected cloned Label"),
    }

    // ---- Goto { label, line } ----
    let goto = find_line(
        &program,
        |l| matches!(l, Line::Goto { label, .. } if label == "loop"),
    );
    let (orig_label, orig_line) = match goto {
        Line::Goto { label, line } => (label.clone(), *line),
        _ => panic!("expected Goto"),
    };
    match goto.clone() {
        Line::Goto { label, line } => {
            assert_eq!(label, "loop");
            assert_eq!(label, orig_label);
            assert_eq!(line, orig_line);
        }
        _ => panic!("expected cloned Goto"),
    }

    // ---- IfGoto { expr, label, line }: operator, label,
    // source line and expression evaluation ----
    let cond = find_line(&program, |l| matches!(l, Line::IfGoto { .. }));
    let (orig_label, orig_line) = match cond {
        Line::IfGoto { label, line, .. } => (label.clone(), *line),
        _ => panic!("expected IfGoto"),
    };
    match cond {
        Line::IfGoto { expr, label, .. } => {
            assert_eq!(label, "loop");
            assert_eq!(expr.operator(), ">");
            assert!(eval_expr(&**expr, 5));
            assert!(!eval_expr(&**expr, 0));
        }
        _ => panic!("expected IfGoto"),
    }
    match cond.clone() {
        Line::IfGoto { expr, label, line } => {
            assert_eq!(label, "loop");
            assert_eq!(label, orig_label);
            assert_eq!(line, orig_line);
            assert_eq!(expr.operator(), ">");
            assert!(eval_expr(&*expr, 5));
            assert!(!eval_expr(&*expr, 0));
        }
        _ => panic!("expected cloned IfGoto"),
    }

    // ---- Assign { name, expr, line } ----
    let assign = find_line(&program, |l| matches!(l, Line::Assign { .. }));
    let (orig_name, orig_line) = match assign {
        Line::Assign { name, line, .. } => (name.clone(), *line),
        _ => panic!("expected Assign"),
    };
    match assign {
        Line::Assign { name, expr, .. } => {
            assert_eq!(name, "energy");
            assert_eq!(expr.operator(), "");
            let ctx = FakeContext::new();
            assert_eq!(
                expr.accept(&mut InterpreterVisitor { ctx: &ctx }).unwrap(),
                Value::Int(5)
            );
        }
        _ => panic!("expected Assign"),
    }
    match assign.clone() {
        Line::Assign { name, expr, line } => {
            assert_eq!(name, "energy");
            assert_eq!(name, orig_name);
            assert_eq!(line, orig_line);
            let ctx = FakeContext::new();
            assert_eq!(
                expr.accept(&mut InterpreterVisitor { ctx: &ctx }).unwrap(),
                Value::Int(5)
            );
        }
        _ => panic!("expected cloned Assign"),
    }

    // ---- Action { action, args, line } ----
    let action_line = find_line(&program, |l| matches!(l, Line::Action { .. }));
    let orig_line = match action_line {
        Line::Action { line, .. } => *line,
        _ => panic!("expected Action"),
    };
    match action_line {
        Line::Action { action, args, .. } => {
            assert_eq!(action.name(), "wait");
            assert_eq!(args.len(), 1);
            let ctx = FakeContext::new();
            assert_eq!(
                args[0]
                    .accept(&mut InterpreterVisitor { ctx: &ctx })
                    .unwrap(),
                Value::Int(1)
            );
        }
        _ => panic!("expected Action"),
    }
    match action_line.clone() {
        Line::Action { action, args, line } => {
            assert_eq!(action.name(), "wait");
            assert_eq!(line, orig_line);
            assert_eq!(args.len(), 1);
            let ctx = FakeContext::new();
            assert_eq!(
                args[0]
                    .accept(&mut InterpreterVisitor { ctx: &ctx })
                    .unwrap(),
                Value::Int(1)
            );
        }
        _ => panic!("expected cloned Action"),
    }
}
