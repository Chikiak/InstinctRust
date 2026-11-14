//! Action tests: I5 (action argument eval error, multiple says in one turn).
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use instinct_core::interp::{execute_turn, CreatureTurnState};
use instinct_core::value::Value;
use instinct_core::world::CreatureState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_action_argument_eval_error_is_runtime_error() {
    // `say(1 / 0)`: the interpreter evaluates the arguments before returning
    // the pending action, and argument evaluation error is a runtime error.
    let (program, _cat) = compile_body("say(1 / 0)\n");
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
    assert!(
        result.is_err(),
        "error evaluating arguments must be a runtime error"
    );
}

#[test]
fn test_multiple_says_in_one_turn() {
    // Three says in a row and then end of file. Each `execute_turn` delivers
    // a pending action (say does not consume a turn), as the simulator does.
    let (program, _cat) = compile_body("say(1)\nsay(2)\nsay(3)\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let pending = run_once(&program, &ctx, &mut state).expect("say(1) pending");
    assert_eq!(pending.action.name(), "say");
    assert_eq!(pending.args, vec![Value::Int(1)]);
    assert_eq!(state.pc, 2);

    let pending = run_once(&program, &ctx, &mut state).expect("say(2) pending");
    assert_eq!(pending.action.name(), "say");
    assert_eq!(pending.args, vec![Value::Int(2)]);
    assert_eq!(state.pc, 3);

    let pending = run_once(&program, &ctx, &mut state).expect("say(3) pending");
    assert_eq!(pending.action.name(), "say");
    assert_eq!(pending.args, vec![Value::Int(3)]);
    assert_eq!(state.pc, 4);

    // End of file: restart at start, no action.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.pc, 0);
}
