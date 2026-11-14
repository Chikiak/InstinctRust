//! Simulation: a world plus turn rules. Advances one tick at a time.
//!
//! The interpreter reads the world immutably; the simulation applies the
//! pending action (and the end-of-tick laws) mutably. A single seeded
//! generator (inside the world) drives both `random` and the turn order, so
//! equal seeds produce identical histories.

mod death;
mod turn;

use crate::event::Event;
use crate::world::{CreatureId, CreatureState, World};

/// A simulation: a world and its turn rules. The world is the single owner of
/// the terrain and object catalogs, so damage and end-of-tick regeneration
/// read and write the same reserve. The seeded generator lives inside the
/// world and is shared by `random` and the turn-order shuffle.
pub struct Simulation {
    pub world: World,
}

impl Simulation {
    /// Creates a simulation from a world that already owns its resource
    /// catalogs. `Simulation` keeps no copies of the catalogs.
    pub fn new(world: World) -> Self {
        Simulation { world }
    }

    /// Seeds the generator. Two simulations with the same seed and the same
    /// placements produce identical histories.
    pub fn set_rng_seed(&mut self, seed: u64) {
        self.world.set_rng_seed(seed);
    }

    /// Returns the current tick count.
    pub fn tick(&self) -> u64 {
        self.world.tick()
    }

    /// Returns an immutable view of the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Advances one full tick: shuffles alive creature ids, runs each turn,
    /// applies end-of-tick laws, and increments the tick counter. Returns all
    /// events emitted during the tick.
    pub fn step_tick(&mut self) -> Vec<Event> {
        // 1. Snapshot the alive creature ids from the store (no linear search
        //    or dead_ids.contains) and shuffle them with the world's
        //    generator. The order is fixed for this tick.
        let mut ids: Vec<CreatureId> = self.world.live_creature_ids();
        let mut order: Vec<usize> = (0..ids.len()).collect();
        self.world.rng.borrow_mut().shuffle(&mut order);
        ids = order.into_iter().map(|i| ids[i]).collect();

        // 2. Run each creature's turn. Awake creatures execute; Sleeping
        //    creatures have their sleep decremented without executing.
        for id in ids {
            let creature = match self.world.creature(id) {
                Some(c) => c.clone(),
                None => continue, // died earlier this tick
            };

            match creature.state {
                CreatureState::Awake => {
                    self.run_creature_turn(id);
                }
                CreatureState::Sleeping(n) => {
                    // Advance sleep by one tick without executing lines.
                    if let Some(c) = self.world.creature_mut(id) {
                        c.state = if n == 1 {
                            CreatureState::Awake
                        } else {
                            CreatureState::Sleeping(n - 1)
                        };
                    }
                }
            }
        }

        // 3. End of tick: creature deaths (-1 health, +1 age, death check).
        let died = self.world.end_of_tick_creatures();
        for event in died {
            self.world.emit_event(event);
        }

        // 4. End of tick: terrain regeneration and object removal on the
        //    world's catalogs (the same reserve mutated by damage).
        self.world.end_of_tick_terrain_and_objects();

        // 5. Rotate roars: the current tick's roars become "previous tick"
        //    for the next one (F2 reads them with roars_near/last_roar
        //    filtering by current_tick - 1).
        self.world.rotate_roars();

        // 6. Advance the tick counter.
        self.world.advance_tick();

        // 7. Drain the event log.
        self.world.drain_events()
    }
}
