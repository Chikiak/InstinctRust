//! `reproduce(dx, dy, health)`: creates an offspring of the same species in
//! the adjacent cell (x+dx, y+dy) of the mother if inside the map and it is
//! only terrain (`CellContent::Empty`) and the mother has health strictly
//! greater than `health`. The mother loses `health`; the offspring is born
//! with health = `health`, age 0, no sleep turns, pc at start (`start:`)
//! and empty variables. Its first turn is the next tick (the `step_tick`
//! snapshot excludes it from the current tick). Any failure is a natural
//! abort: it does nothing, but consumes the turn.

use crate::actions::shared::{
    check_arity, checked_sub_health, creature_context, is_out_of_bounds, noop_result,
    parse_dir_arg, parse_int_arg, unknown_creature_diagnostic, ACTION_REPRODUCE, ARG_DX, ARG_DY,
    ARG_HEALTH,
};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reproduce;

impl crate::actions::Action for Reproduce {
    fn name(&self) -> &str {
        ACTION_REPRODUCE
    }
    fn arity(&self) -> usize {
        3
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        world: &mut dyn ActionContext,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        check_arity(ACTION_REPRODUCE, self.arity(), args.len())?;
        // --- Phase 1: reads and pure computations only (nothing mutated) ---
        let dx = parse_dir_arg(&args[0], ACTION_REPRODUCE, ARG_DX)?;
        let dy = parse_dir_arg(&args[1], ACTION_REPRODUCE, ARG_DY)?;
        let health = parse_int_arg(&args[2], ACTION_REPRODUCE, ARG_HEALTH)?;

        let (pos, _species, _) = creature_context(world, creature_id)?;

        let mother = world
            .creature(creature_id)
            .ok_or_else(|| unknown_creature_diagnostic(&creature_id))?;
        if mother.health <= health {
            return Ok(noop_result());
        }
        let species = mother.species.clone();
        let mother_start_health = mother.health;

        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;
        let new_mother_health =
            checked_sub_health(mother_start_health, health, ACTION_REPRODUCE, pos)?;

        if is_out_of_bounds(world, tx, ty) {
            return Ok(noop_result());
        }
        let (tx, ty) = (tx as usize, ty as usize);

        // --- Fase 2: escrituras ---
        let child_id = match world.place_creature(&species, tx, ty) {
            Ok(id) => id,
            Err(_) => return Ok(noop_result()),
        };
        // If we cannot assign health to the child, revert creation so as not to
        // leave an orphaned, health-less child. If the revert also fails,
        // combine both errors to not lose information.
        if let Err(e) = world.set_creature_health(child_id, health) {
            if let Err(revert_err) = world.remove_creature(child_id) {
                return Err(Diagnostic::new(
                    ACTION_REPRODUCE,
                    crate::actions::consts::DIAG_LINE,
                    format!(
                        "failed to set child health: {e}; failed to revert child creation: {revert_err}"
                    ),
                ));
            }
            return Err(e);
        }

        let mother = world
            .creature_mut(creature_id)
            .ok_or_else(|| unknown_creature_diagnostic(&creature_id))?;
        mother.health = new_mother_health;

        Ok(noop_result())
    }
}
