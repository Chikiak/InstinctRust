//! `Action` trait and `ActionResult`: the interpreter applies actions through
//! this trait and the world applies the returned action.

use crate::diagnostic::Diagnostic;
use crate::event::Event;
use crate::value::Value;
use crate::world::{CreatureId, World};

/// Result of applying an action: the number of sleep turns (0 = awake) and
/// any events to append to the world's event log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub sleep_turns: u32,
    pub events: Vec<Event>,
}

/// Trait every action type implements. Stateless: no per-creature state here.
pub trait Action: std::fmt::Debug {
    /// Action name as it appears in the language (e.g. `wait`, `move`, `say`).
    fn name(&self) -> &str;
    /// Number of evaluated arguments the action expects.
    fn arity(&self) -> usize;
    /// Whether executing this action consumes a turn. `say` does not.
    fn consumes_turn(&self) -> bool;
    /// Apply the action with already-evaluated arguments on the world for the
    /// given creature id. Returns an `ActionResult`.
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic>;
}
