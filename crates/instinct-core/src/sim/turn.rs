//! Runs a single creature's turn.

use crate::interp::{execute_turn, CreatureTurnState};
use crate::program::Program;
use crate::world::{CreatureContext, CreatureId};
use std::cell::RefCell;
use std::rc::Rc;

impl super::Simulation {
    /// Runs a single creature's turn. The turn loops while `say` actions are
    /// executed (they don't consume the turn). A runtime error kills the
    /// creature.
    ///
    /// A `CreatureMemento` is captured at the start of the turn. If the turn
    /// fails with a runtime error, the creature state is restored from the
    /// memento and then the creature dies cleanly (F2/F3 semantics), so the
    /// world never ends up in a partially corrupted intermediate state.
    pub(in crate::sim) fn run_creature_turn(&mut self, id: CreatureId) {
        // Snapshot the creature (owned data, no borrow of self.world held).
        let creature = match self.world.creature(id).cloned() {
            Some(c) => c,
            None => return, // died earlier this tick
        };

        // Memento of the state at the start of the turn (Memento pattern).
        let memento = creature.memento();

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
                    // Restore state from the memento before dying, to
                    // guarantee no partial changes remain in the world.
                    let _ = self.world.restore_creature_from_memento(&memento);
                    self.die_by_error(id, &creature, diagnostic.message, line);
                    return;
                }
            };

            // Persist the updated turn state back to the creature.
            if let Some(c) = self.world.creature_mut(id) {
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
}
