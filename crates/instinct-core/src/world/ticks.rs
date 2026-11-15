//! Timing, events, RNG, and output port for the world.

use super::creature::{CreatureId, CreatureState};
use super::World;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::event::{Event, WorldOutput};
use crate::rng::{Rng, RngSource};

impl World {
    /// Seeds the world's generator. Two worlds with the same seed produce the
    /// same `random` sequence and turn order.
    pub fn set_rng_seed(&mut self, seed: u64) {
        *self.rng.borrow_mut() = Box::new(Rng::new(seed));
    }

    /// Injects a `RngSource` strategy (e.g. a deterministic `TestRng` for
    /// tests). Replaces the world's current generator.
    pub fn set_rng_source(&mut self, rng: Box<dyn RngSource>) {
        *self.rng.borrow_mut() = rng;
    }

    /// Connects an output port to the world. Events emitted by `emit_event`
    /// are forwarded to the port in addition to being stored in `self.events`.
    /// The port is optional: if nothing is connected, the world works as
    /// before. A port error never propagates to the tick.
    pub fn set_output(&mut self, output: Box<dyn WorldOutput>) {
        self.output = Some(output);
    }

    /// Returns the current tick count.
    pub fn tick(&self) -> u64 {
        self.ticks
    }

    /// Advances the tick counter by one.
    pub fn advance_tick(&mut self) {
        self.ticks += 1;
    }

    /// Appends an event to the world's event log and forwards it to the
    /// output port if one is connected. A failure in the port never aborts
    /// the tick.
    pub fn emit_event(&mut self, event: Event) {
        // Forward to the port before storing: the port borrows a reference to
        // the event, then the event is moved into the log.
        if let Some(output) = self.output.as_mut() {
            output.emit(&event);
        }
        self.events.push(event);
    }

    /// Drains all pending events, returning them and leaving the log empty.
    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Applies the result of an action: sets the creature's turn state and
    /// appends events. `sleep_turns > 0` means `Sleeping(sleep_turns)`;
    /// zero means `Awake`.
    pub fn apply_action_result(
        &mut self,
        creature_id: CreatureId,
        result: ActionResult,
    ) -> Result<(), Diagnostic> {
        if let Some(creature) = self.creature_mut(creature_id) {
            creature.state = if result.sleep_turns > 0 {
                CreatureState::Sleeping(result.sleep_turns)
            } else {
                CreatureState::Awake
            };
        } else {
            return Err(Diagnostic::new(
                "world",
                0,
                format!("unknown creature id {creature_id:?}"),
            ));
        }
        for event in result.events {
            self.emit_event(event);
        }
        Ok(())
    }
}
