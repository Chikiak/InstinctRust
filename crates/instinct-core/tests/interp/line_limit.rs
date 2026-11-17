//! Line limit tests: AC2 (line limit ends a turn) and AC1 (limit counts
//! executed lines; pc stays valid).
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
fn test_line_limit_ends_turn() {
    // start:(0), loop:(1), goto loop:(2)  -> 2 lines per iteration after start
    let (program, _cat) = compile_body("loop:\ngoto loop\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    let pending = run_turn_full(&program, &ctx, &mut state);
    assert!(
        pending.is_none(),
        "line limit should end the turn with no action"
    );
    // pc left where the limit was hit (index 2, the goto line).
    assert_eq!(state.pc, 2);
}

#[test]
fn test_line_limit_executes_100_and_pc_valid() {
    // 152 lines: label start + `n = 0` + 150 additions.
    // The turn should stop at line 100 (index 100), valid and not executed.
    let mut body = String::from("n = 0\n");
    for i in 0..150 {
        body.push_str(&format!("n = n + {i}\n"));
    }
    let (program, _cat) = compile_body(&body);
    assert_eq!(program.lines.len(), 152);

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
    assert!(pending.is_none(), "limit reached with no pending action");
    // Exactly 100 lines executed: label start, `n = 0`, and 98 additions
    // (i = 0..97), so n = 0 + 0 + ... + 97 = 4753.
    assert_eq!(state.pc, 100, "pc points to the first unexecuted line");
    assert_eq!(
        ctx.vars.borrow().get("n"),
        Some(&Value::Int(4753)),
        "only 98 additions were executed"
    );
}
