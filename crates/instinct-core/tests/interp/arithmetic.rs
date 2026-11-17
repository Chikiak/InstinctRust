//! Arithmetic tests: AC4 (arithmetic extremes are runtime errors, never panic).
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use instinct_core::interp::{execute_turn, CreatureTurnState};
use instinct_core::world::CreatureState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_arithmetic_extremes_are_runtime_errors() {
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };

    // Overflow: i64::MAX + 1.
    let (program, _cat) = compile_body("n = 9223372036854775807 + 1\n");
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };
    let (d, _line) = execute_turn(&program, &ctx, &mut state).unwrap_err();
    assert!(
        d.message.contains("overflow"),
        "sum overflow: {:?}",
        d.message
    );

    // Remainder by zero.
    let (program, _cat) = compile_body("n = 7 % 0\n");
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };
    let (d, _line) = execute_turn(&program, &ctx, &mut state).unwrap_err();
    assert!(
        d.message.contains("modulo by zero"),
        "remainder by zero: {:?}",
        d.message
    );

    // Negation of i64::MIN: m = i64::MIN (without overflowing when constructing it),
    // then -m overflows.
    let (program, _cat) = compile_body("m = -9223372036854775807 - 1\nn = -m\n");
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };
    let (d, line) = execute_turn(&program, &ctx, &mut state).unwrap_err();
    assert_eq!(line, 8, "the error is on the second line of the body");
    assert!(
        d.message.contains("unary minus"),
        "negation of i64::MIN: {:?}",
        d.message
    );
}
