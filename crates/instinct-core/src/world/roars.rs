//! Roars: shouts emitted by creatures, associated with species, position, and text.

use super::creature::CreatureId;
use super::World;
use crate::event::Pos;

/// A roar emitted by a creature in a tick, associated with its species,
/// position, and text. Stored with its tick so that, in the next tick, F2 can
/// read previous-tick roars with `roars_near`/`last_roar` within vision radius.
/// `creature_id` identifies the emitter so a creature does not count or read
/// its own roar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roar {
    pub tick: u64,
    pub creature_id: CreatureId,
    pub species: String,
    pub pos: Pos,
    pub text: String,
}

impl World {
    /// Registers a roar emitted by creature `creature_id` from `pos` with text
    /// `text` in the current tick. `roar` does not kill or damage: it only
    /// records the message so F2 can read it the next tick with
    /// `roars_near`/`last_roar`.
    pub fn add_roar(&mut self, creature_id: CreatureId, species: String, pos: Pos, text: String) {
        self.roars_current.push(Roar {
            tick: self.ticks,
            creature_id,
            species,
            pos,
            text,
        });
    }

    /// Returns roars from the previous tick (available during the current
    /// tick). Empty until `sim.rs` performs the first rotation.
    pub fn previous_roars(&self) -> &[Roar] {
        &self.roars_previous
    }

    /// Marks current-tick roars as "previous tick" and clears current ones.
    /// `sim.rs` calls this at the end of each tick so a roar does not persist
    /// beyond the tick after its emission.
    pub fn rotate_roars(&mut self) {
        self.roars_previous = std::mem::take(&mut self.roars_current);
    }
}
