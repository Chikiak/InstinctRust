//! Wait tests: AC4 (wait(3) skips two turns).
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use instinct_core::interp::CreatureTurnState;
use instinct_core::value::Value;
use instinct_core::world::CreatureState;

#[test]
fn test_wait_skips_turns() {
    // start:(0), wait(3):(1), goto start:(2)
    let (program, _cat) = compile_body("wait(3)\ngoto start\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    // Turn 1: wait(3) -> pending action. Sleep state set by applying the action.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "wait");
    assert_eq!(pa.args, vec![Value::Int(3)]);
    state.state = CreatureState::Sleeping(2); // simulate apply_action_result setting sleep
    assert_eq!(state.state, CreatureState::Sleeping(2));

    // Turn 2: sleep 2 -> decrement to 1, no action.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.state, CreatureState::Sleeping(1));

    // Turn 3: sleep 1 -> decrement to 0, no action.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.state, CreatureState::Awake);

    // Turn 4: awake. pc = 2 (goto start) -> goto sets pc to 0, then wait(3) -> sleep 2.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "wait");
    state.state = CreatureState::Sleeping(2); // simulate apply_action_result
    assert_eq!(state.state, CreatureState::Sleeping(2));
}
