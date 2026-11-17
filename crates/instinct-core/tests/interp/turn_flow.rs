//! Turn flow tests: AC1 (labels, goto, if, assign, say without consuming turn;
//! stops at first non-say action; resumes at the next line).
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use instinct_core::interp::CreatureTurnState;
use instinct_core::value::Value;
use instinct_core::world::CreatureState;

#[test]
fn test_turn_resumes_after_action() {
    // start:(0), n = 5:(1), say(n):(2), move(1,0,1):(3), goto start:(4)
    let (program, _cat) = compile_body("n = 5\nsay(n)\nmove(1, 0, 1)\ngoto start\n");

    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    // Turn 1: assign n=5, say(n) (no turn consumed), move ends turn.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(pa.args, vec![Value::Int(1), Value::Int(0), Value::Int(1)]);
    // pc left after the move line (index 3 -> 4); next turn resumes at goto (index 4).
    assert_eq!(state.pc, 4);
    assert_eq!(ctx.vars.borrow().get("n"), Some(&Value::Int(5)));

    // Turn 2: goto start (sets pc to 0), then n=5, say(n), move ends turn.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(state.pc, 4);
}
