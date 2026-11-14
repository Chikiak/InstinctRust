//! Shared test helpers for the interpreter tests.
//! Extracted from the top of `tests/interp_tests.rs`: SharedOutput,
//! compile_body, FakeCtx, run_once, and run_turn_full. Logic is identical to
//! the original; comments were translated to English. Items are `pub` so
//! sibling submodules can import them via `use super::common::*`.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::{Event, WorldOutput};
use instinct_core::interp::{execute_turn, CreatureTurnState, PendingAction};
use instinct_core::perceptions::PerceptionCatalog;
use instinct_core::program::Program;
use instinct_core::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Shares an event Vec between the output port and the test via `Rc<RefCell>`,
/// so the test can verify the port received the event.
pub struct SharedOutput {
    events: Rc<RefCell<Vec<Event>>>,
}

impl WorldOutput for SharedOutput {
    fn emit(&mut self, event: &Event) {
        self.events.borrow_mut().push(event.clone());
    }
}

/// Compiles a minimal `.ins` body (header + `start:`) with the given body
/// lines.
pub fn compile_body(body: &str) -> (Program, Catalog) {
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
pub struct FakeCtx {
    vars: Rc<RefCell<HashMap<String, Value>>>,
}

impl instinct_core::expr::EvalContext for FakeCtx {
    fn resolve(&self, name: &str) -> Option<Value> {
        self.vars.borrow().get(name).cloned()
    }
}

/// Runs `execute_turn` once. Returns the pending action and the new state.
pub fn run_once(
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
pub fn run_turn_full(
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
