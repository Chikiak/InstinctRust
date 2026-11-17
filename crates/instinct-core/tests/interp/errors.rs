//! Error tests: division by zero, unassigned variable, missing start, out-of-range
//! goto/if-goto, and unknown label.
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use instinct_core::expr::LiteralInt;
use instinct_core::interp::{execute_turn, CreatureTurnState};
use instinct_core::program::{Line, Program};
use instinct_core::world::CreatureState;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

#[test]
fn test_division_by_zero_is_runtime_error() {
    // start:(6), n = 1 / 0:(7)
    let (program, _cat) = compile_body("n = 1 / 0\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err());
    let (_d, line) = result.unwrap_err();
    assert_eq!(line, 7); // `n = 1 / 0` is on line 7
}

#[test]
fn test_unassigned_variable_is_runtime_error() {
    let (program, _cat) = compile_body("n = m + 1\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err());
}

#[test]
fn test_execute_turn_missing_start_errors() {
    // Hand-built program: one line with label "foo", without "start".
    let program = Program {
        lines: vec![Line::Label {
            name: "foo".to_string(),
            line: 1,
        }],
        labels: BTreeMap::from([("foo".to_string(), 0)]),
    };
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err(), "execute_turn must fail without start");
    let (d, _line) = result.unwrap_err();
    assert!(
        d.message.contains("start"),
        "error names the missing label: {d:?}"
    );
}

#[test]
fn test_goto_out_of_range_errors() {
    // `loop` is in the label table but points to index 5 of a two-line
    // program: only reachable if the program is mutated after compiling.
    // The jump must fail naming the label.
    let program = Program {
        lines: vec![
            Line::Label {
                name: "start".to_string(),
                line: 1,
            },
            Line::Goto {
                label: "loop".to_string(),
                line: 2,
            },
        ],
        labels: BTreeMap::from([("start".to_string(), 0), ("loop".to_string(), 5)]),
    };
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
    assert!(result.is_err(), "out-of-range goto must be a runtime error");
    let (d, line) = result.unwrap_err();
    assert_eq!(line, 2);
    assert!(
        d.message.contains("loop"),
        "the error names the label: {d:?}"
    );
    // pc was never corrupted: no out-of-range indexing.
    assert_eq!(state.pc, 1);
}

#[test]
fn test_if_goto_out_of_range_errors() {
    // `if 1 == 1 goto far` with `far` out of range: error when jumping.
    let program = Program {
        lines: vec![
            Line::Label {
                name: "start".to_string(),
                line: 1,
            },
            Line::IfGoto {
                expr: Box::new(LiteralInt { value: 1 }) as Box<dyn instinct_core::expr::Expr>,
                label: "far".to_string(),
                line: 2,
            },
        ],
        labels: BTreeMap::from([("start".to_string(), 0), ("far".to_string(), 9)]),
    };
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
        "if goto out of range must be a runtime error"
    );
    let (d, line) = result.unwrap_err();
    assert_eq!(line, 2);
    assert!(
        d.message.contains("far"),
        "the error names the label: {d:?}"
    );
    assert_eq!(state.pc, 1);
}

#[test]
fn test_goto_unknown_label_is_runtime_error() {
    // Hand-built program: `goto missing` without `missing` in the table.
    let program = Program {
        lines: vec![
            Line::Label {
                name: "start".to_string(),
                line: 1,
            },
            Line::Goto {
                label: "missing".to_string(),
                line: 2,
            },
        ],
        labels: BTreeMap::from([("start".to_string(), 0)]),
    };
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
    assert!(result.is_err(), "goto to unknown label must fail");
    let (d, line) = result.unwrap_err();
    assert_eq!(line, 2);
    assert!(
        d.message.contains("missing"),
        "the error names the label: {d:?}"
    );
}
