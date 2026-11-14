//! Turn state and results for a single creature execution.

use crate::actions::Action;
use crate::value::Value;
use crate::world::CreatureState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// The state of a creature's turn, kept between turns by the simulation.
#[derive(Debug, Clone)]
pub struct CreatureTurnState {
    /// Next line index to execute.
    pub pc: usize,
    /// Turn cycle state: Awake or Sleeping(n). Replaces `sleep_turns`.
    pub state: CreatureState,
    /// Variables owned by this creature. Shared via `Rc<RefCell>` so the
    /// evaluation context can see assignments made during the same turn.
    pub variables: Rc<RefCell<HashMap<String, Value>>>,
}

impl CreatureTurnState {
    pub fn new() -> Self {
        CreatureTurnState {
            pc: 0,
            state: CreatureState::Awake,
            variables: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl Default for CreatureTurnState {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of executing a turn.
#[derive(Debug, Clone)]
pub struct TurnResult {
    /// The pending action, if the turn ended on an action.
    pub pending: Option<PendingAction>,
    /// The turn state after execution (pc, sleep, variables).
    pub state: CreatureTurnState,
}

/// A pending action waiting to be applied by the world, with its arguments
/// already evaluated.
#[derive(Debug, Clone)]
pub struct PendingAction {
    pub action: Rc<dyn Action>,
    pub args: Vec<Value>,
    /// Source line number of the action (for RuntimeError reporting).
    pub line: usize,
}
