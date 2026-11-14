//! IfGoto tests: I5 (true, false, and evaluation error).
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use instinct_core::expr::{Expr, LiteralInt};
use instinct_core::interp::CreatureTurnState;
use instinct_core::value::Value;
use instinct_core::world::CreatureState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_if_goto_true_jumps() {
    // `if a == 1 goto done` jumps over `n = 2`; end of file restarts.
    let (program, _cat) = compile_body("a = 1\nif a == 1 goto done\nn = 2\ndone:\nn = 3\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let pending = run_turn_full(&program, &ctx, &mut state);
    assert!(pending.is_none(), "no action at end of program");
    assert_eq!(state.pc, 0, "restart at start after end of file");
    assert_eq!(
        ctx.vars.borrow().get("a"),
        Some(&Value::Int(1)),
        "the assignment before the jump was executed"
    );
    assert_eq!(
        ctx.vars.borrow().get("n"),
        Some(&Value::Int(3)),
        "the middle line was skipped and the one after the label was executed"
    );
}

#[test]
fn test_if_goto_false_falls_through() {
    // `if 1 == 2 goto done` does not jump; pc moves to the next line.
    let (program, _cat) = compile_body("if 1 == 2 goto done\nn = 5\ndone:\nwait(1)\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "wait");
    assert_eq!(pa.args, vec![Value::Int(1)]);
    assert_eq!(state.pc, 5, "pc advances line by line without jumping");
    assert_eq!(
        ctx.vars.borrow().get("n"),
        Some(&Value::Int(5)),
        "the line after the if was executed"
    );
}

#[test]
fn test_if_goto_eval_error_is_runtime_error() {
    // A condition with an evaluation error propagates the error, no panic.
    let (program, _cat) = compile_body("if 1 / 0 goto lab\nlab:\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err(), "if goto with error must fail");
}
