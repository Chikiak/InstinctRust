//! Unit tests for the turn interpreter (interp.rs).
//! Programs are compiled from text in memory and executed turn by turn.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::interp::{execute_turn, CreatureTurnState, PendingAction};
use instinct_core::perception::PerceptionCatalog;
use instinct_core::program::Program;
use instinct_core::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Compiles a minimal `.ins` body (header + start:) with the given body lines.
fn compile_body(body: &str) -> (Program, Catalog) {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text =
        format!("creature Test\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\n{body}");
    let result = compile::compile(&text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    (result.program, cat)
}

/// A fake evaluation context backed by a shared variable map. Assignments made
/// during a turn are visible to later reads in the same turn.
struct FakeCtx {
    vars: Rc<RefCell<HashMap<String, Value>>>,
}

impl instinct_core::expr::EvalContext for FakeCtx {
    fn resolve(&self, name: &str) -> Option<Value> {
        self.vars.borrow().get(name).cloned()
    }
}

/// Runs `execute_turn` once. Returns the pending action and the new state.
fn run_once(
    program: &Program,
    ctx: &FakeCtx,
    state: &mut CreatureTurnState,
) -> Option<PendingAction> {
    match execute_turn(program, ctx, state) {
        Ok(result) => {
            *state = result.state;
            result.pending
        }
        Err((d, line)) => panic!("runtime error at line {line}: {:?}", d),
    }
}

/// Runs a full turn: loops while `say` actions are returned (they don't
/// consume the turn), stops at the first non-say pending action or when no
/// action is returned. Returns the final non-say pending action (if any).
fn run_turn_full(
    program: &Program,
    ctx: &FakeCtx,
    state: &mut CreatureTurnState,
) -> Option<PendingAction> {
    loop {
        match run_once(program, ctx, state)? {
            pending if pending.action.consumes_turn() => return Some(pending),
            _ => continue, // say: applied by simulation, continue turn
        }
    }
}

// ---------------------------------------------------------------------------
// AC1: labels, goto, if, assign, say without consuming turn; stops at first
// non-say action; resumes at the next line.
// ---------------------------------------------------------------------------

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
        sleep_turns: 0,
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

// ---------------------------------------------------------------------------
// AC2: line limit - a loop of label + goto ends at 100 lines.
// ---------------------------------------------------------------------------

#[test]
fn test_line_limit_ends_turn() {
    // start:(0), loop:(1), goto loop:(2)  -> 2 lines per iteration after start
    let (program, _cat) = compile_body("loop:\ngoto loop\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        sleep_turns: 0,
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

// ---------------------------------------------------------------------------
// AC3: end of file -> restart at start.
// ---------------------------------------------------------------------------

#[test]
fn test_end_of_program_restarts_at_start() {
    // start:(0), move(1,0,1):(1)  -> end of file after move
    let (program, _cat) = compile_body("move(1, 0, 1)\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        sleep_turns: 0,
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

// ---------------------------------------------------------------------------
// AC4: wait(3) skips two turns.
// ---------------------------------------------------------------------------

#[test]
fn test_wait_skips_turns() {
    // start:(0), wait(3):(1), goto start:(2)
    let (program, _cat) = compile_body("wait(3)\ngoto start\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        sleep_turns: 0,
        variables: Rc::clone(&ctx.vars),
    };

    // Turn 1: wait(3) -> pending action. Sleep turns set by applying the action.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "wait");
    assert_eq!(pa.args, vec![Value::Int(3)]);
    state.sleep_turns = 2; // simulate apply_action_result setting sleep_turns
    assert_eq!(state.sleep_turns, 2);

    // Turn 2: sleep 2 -> decrement to 1, no action.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.sleep_turns, 1);

    // Turn 3: sleep 1 -> decrement to 0, no action.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.sleep_turns, 0);

    // Turn 4: awake. pc = 2 (goto start) -> goto sets pc to 0, then wait(3) -> sleep 2.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "wait");
    state.sleep_turns = 2; // simulate apply_action_result
    assert_eq!(state.sleep_turns, 2);
}

// ---------------------------------------------------------------------------
// AC7: division by zero -> runtime error.
// ---------------------------------------------------------------------------

#[test]
fn test_division_by_zero_is_runtime_error() {
    // start:(6), n = 1 / 0:(7)
    let (program, _cat) = compile_body("n = 1 / 0\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        sleep_turns: 0,
        variables: Rc::clone(&ctx.vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err());
    let (_d, line) = result.unwrap_err();
    assert_eq!(line, 7); // `n = 1 / 0` is on line 7
}

// ---------------------------------------------------------------------------
// Variable read before assign -> runtime error.
// ---------------------------------------------------------------------------

#[test]
fn test_unassigned_variable_is_runtime_error() {
    let (program, _cat) = compile_body("n = m + 1\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        sleep_turns: 0,
        variables: Rc::clone(&ctx.vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err());
}
