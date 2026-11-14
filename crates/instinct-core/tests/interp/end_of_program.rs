//! End-of-program tests: AC3 (end of file -> restart at start).
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use instinct_core::interp::CreatureTurnState;
use instinct_core::world::CreatureState;

#[test]
fn test_end_of_program_restarts_at_start() {
    // start:(0), move(1,0,1):(1)  -> end of file after move
    let (program, _cat) = compile_body("move(1, 0, 1)\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    // Turn 1: move ends the turn. pc left after move (index 1 -> 2, end of file).
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(state.pc, 2);

    // Turn 2: end of file -> restart at start, no action.
    let pending = run_turn_full(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.pc, 0);

    // Turn 3: move again from start.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(state.pc, 2);
}
