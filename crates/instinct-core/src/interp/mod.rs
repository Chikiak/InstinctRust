//! Execution of a single creature turn.
//!
//! The interpreter reads the world immutably (through `CreatureContext`) and
//! evaluates the compiled `Program` starting from the creature's saved pc.
//! It returns at most one pending action (`Option<PendingAction>`); the world
//! applies it afterwards. `say` is returned as a pending action too, and the
//! simulation applies it in the act without ending the turn.

pub mod state;
pub mod visitor;
pub mod lines;
pub mod turn;

pub use state::{CreatureTurnState, TurnResult, PendingAction};
pub use visitor::InterpreterVisitor;
pub use lines::{execute_line, jump_target};
pub use turn::execute_turn;
