//! Simulation: a world plus turn rules. Advances one tick at a time.
//!
//! The interpreter reads the world immutably; the simulation applies the
//! pending action (and the end-of-tick laws) mutably. A single seeded
//! generator (inside the world) drives both `random` and the turn order, so
//! equal seeds produce identical histories.

use crate::event::{Event, Pos};
use crate::interp::{execute_turn, CreatureTurnState};
use crate::program::Program;
use crate::world::{CreatureContext, CreatureId, CreatureState, World};
use std::cell::RefCell;
use std::rc::Rc;

/// A simulation: a world and its turn rules. The world is the single owner of
/// the terrain and object catalogs, so damage and end-of-tick regeneration
/// read and write the same reserve. The seeded generator lives inside the
/// world and is shared by `random` and the turn-order shuffle.
pub struct Simulation {
    pub world: World,
}

impl Simulation {
    /// Creates a simulation from a world that already owns its resource
    /// catalogs. `Simulation` no guarda copias propias de los catálogos.
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
        // 1. Snapshot the alive creature ids and shuffle them with the world's
        //    generator. The order is fixed for this tick.
        let mut ids: Vec<CreatureId> = self.world.creatures.iter().map(|c| c.id).collect();
        let mut order: Vec<usize> = (0..ids.len()).collect();
        self.world.rng.borrow_mut().shuffle(&mut order);
        ids = order.into_iter().map(|i| ids[i]).collect();

        // 2. Run each creature's turn. Awake creatures execute; Sleeping
        //    creatures have their sleep decremented without executing.
        for id in ids {
            let creature = match self.world.creatures.iter().find(|c| c.id == id) {
                Some(c) => c.clone(),
                None => continue, // died earlier this tick
            };

            match creature.state {
                CreatureState::Awake => {
                    self.run_creature_turn(id);
                }
                CreatureState::Sleeping(n) => {
                    // Advance sleep by one tick without executing lines.
                    if let Some(c) = self.world.creatures.iter_mut().find(|c| c.id == id) {
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

        // 4. End of tick: terrain regeneration and object removal sobre los
        //    catálogos del mundo (misma reserva que el daño mutó).
        self.world.end_of_tick_terrain_and_objects();

        // 5. Rotar rugidos: los del tick actual pasan a ser "del tick
        //    anterior" para el siguiente (F2 los lee con roars_near/last_roar
        //    filtrando por tick_actual - 1).
        self.world.rotate_roars();

        // 6. Advance the tick counter.
        self.world.advance_tick();

        // 6. Drain the event log.
        self.world.drain_events()
    }

    /// Runs a single creature's turn. The turn loops while `say` actions are
    /// executed (they don't consume the turn). A runtime error kills the
    /// creature.
    fn run_creature_turn(&mut self, id: CreatureId) {
        // Snapshot the creature (owned data, no borrow of self.world held).
        let creature = match self.world.creatures.iter().find(|c| c.id == id) {
            Some(c) => c.clone(),
            None => return, // died earlier this tick
        };

        let program: &Program = match &creature.species.program {
            Some(p) => p.as_ref(),
            None => {
                // No compiled program: creature cannot act.
                return;
            }
        };

        let mut state = CreatureTurnState {
            pc: creature.pc,
            state: creature.state.clone(),
            variables: Rc::new(RefCell::new(creature.variables.clone())),
        };

        loop {
            let ctx = CreatureContext::new(&self.world, id, state.variables.clone());
            let result = match execute_turn(program, &ctx, &mut state) {
                Ok(r) => r,
                Err((diagnostic, line)) => {
                    self.die_by_error(id, &creature, diagnostic.message, line);
                    return;
                }
            };

            // Persist the updated turn state back to the creature.
            if let Some(c) = self.world.creatures.iter_mut().find(|c| c.id == id) {
                c.pc = result.state.pc;
                c.state = result.state.state.clone();
                c.variables = state.variables.borrow().clone();
            } else {
                return;
            }

            match result.pending {
                Some(pending) => {
                    if !pending.action.consumes_turn() {
                        // `say`: apply in the act, does not end the turn.
                        match pending.action.apply(&mut self.world, id, &pending.args) {
                            Ok(action_result) => {
                                let _ = self.world.apply_action_result(id, action_result);
                                // Continue the turn at the next line.
                                continue;
                            }
                            Err(diagnostic) => {
                                self.die_by_error(id, &creature, diagnostic.message, pending.line);
                                return;
                            }
                        }
                    } else {
                        // A turn-ending action: apply and finish the turn.
                        match pending.action.apply(&mut self.world, id, &pending.args) {
                            Ok(action_result) => {
                                let _ = self.world.apply_action_result(id, action_result);
                            }
                            Err(diagnostic) => {
                                self.die_by_error(id, &creature, diagnostic.message, pending.line);
                                return;
                            }
                        }
                        return;
                    }
                }
                None => {
                    // No action (end of file or sleep skipped).
                    return;
                }
            }
        }
    }

    /// Kills a creature due to a runtime error: emits `RuntimeError` and
    /// `Died` (cause `error`) events and removes it from the map.
    fn die_by_error(
        &mut self,
        id: CreatureId,
        creature: &crate::world::Creature,
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
