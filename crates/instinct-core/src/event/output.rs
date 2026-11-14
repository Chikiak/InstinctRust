//! World output port: consumes events from the bus without the core knowing
//! who prints them (CLI, tomorrow the GUI). A port that fails to emit does
//! not abort the tick; the bus ignores the consumer's error.

use crate::event::events::Event;

/// World output port: consumes events from the bus without the core knowing
/// who prints them (CLI, tomorrow the GUI). A port that fails to emit does
/// not abort the tick; the bus ignores the consumer's error.
pub trait WorldOutput {
    /// Receives an event from the world bus. Typical implementers print it,
    /// store it, or send it to a window.
    fn emit(&mut self, event: &Event);
}
