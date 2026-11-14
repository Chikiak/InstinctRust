//! Kills a creature due to a runtime error.

use crate::event::{Event, Pos};
use crate::world::{Creature, CreatureId};

impl super::Simulation {
    /// Kills a creature due to a runtime error: emits `RuntimeError` and
    /// `Died` (cause `error`) events and removes it from the map.
    pub(in crate::sim) fn die_by_error(
        &mut self,
        id: CreatureId,
        creature: &Creature,
        message: String,
        line: usize,
    ) {
        let pos = Pos {
            x: creature.x,
            y: creature.y,
        };
        let species = creature.species.name.clone();
        let tick = self.world.tick();
        self.world.emit_event(Event::RuntimeError {
            tick,
            species: species.clone(),
            pos,
            message,
            line,
        });
        let _ = self.world.remove_creature(id);
        self.world.emit_event(Event::Died {
            tick,
            species,
            pos,
            cause: "error".to_string(),
        });
    }
}
