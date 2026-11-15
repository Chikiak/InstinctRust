//! `reproduce(dx, dy, health)`: creates an offspring of the same species in
//! the adjacent cell (x+dx, y+dy) of the mother if inside the map and it is
//! only terrain (`CellContent::Empty`) and the mother has health strictly
//! greater than `health`. The mother loses `health`; the offspring is born
//! with health = `health`, age 0, no sleep turns, pc at start (`start:`)
//! and empty variables. Its first turn is the next tick (the `step_tick`
//! snapshot excludes it from the current tick). Any failure is a natural
//! abort: it does nothing, but consumes the turn.

use crate::actions::shared::{
    checked_sub_health, creature_context, is_out_of_bounds, noop_result, parse_dir_arg,
    parse_int_arg, ACTION_REPRODUCE, ARG_DX, ARG_DY, ARG_HEALTH, DIAG_WORLD,
};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reproduce;

/// Builds the `Diagnostic` reported when a creature id does not exist
/// in the world. A single site defines it so both borrows
/// (immutable and mutable) share the message without duplicating it.
fn unknown_creature_diagnostic(creature_id: &CreatureId) -> Diagnostic {
    Diagnostic::new(DIAG_WORLD, 0, format!("unknown creature id {creature_id:?}"))
}

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
        // --- parsing ---
        let dx = parse_dir_arg(&args[0], ACTION_REPRODUCE, ARG_DX)?;
        let dy = parse_dir_arg(&args[1], ACTION_REPRODUCE, ARG_DY)?;
        let health = parse_int_arg(&args[2], ACTION_REPRODUCE, ARG_HEALTH)?;

        // --- creature context ---
        let (pos, _species, _tick) = creature_context(world, creature_id)?;
        let mother = world.creature(creature_id)
            .ok_or_else(|| unknown_creature_diagnostic(&creature_id))?;

        // --- validation: mother must have health strictly greater than `health` ---
        if mother.health <= health {
            return Ok(noop_result());
        }
        let species = mother.species.clone();
        let mother_health = mother.health;

        // --- mother health: checked subtraction ---
        let new_mother_health = checked_sub_health(mother_health, health, ACTION_REPRODUCE, pos)?;

        // --- bounds check ---
        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;
        // Out of map: natural abort.
        if is_out_of_bounds(world, tx, ty) {
            return Ok(noop_result());
        }

        // --- place child ---
        // `place_creature` validates bounds and free cell: occupied cell ->
        // natural abort without touching the mother.
        let child_id = match world.place_creature(&species, tx as usize, ty as usize) {
            Ok(id) => id,
            Err(_) => return Ok(noop_result()),
        };
        // `place_creature` initializes with `species.health`; adjust to `health`.
        world.set_creature_health(child_id, health)?;

        // --- apply damage to mother ---
        let mother = world.creature_mut(creature_id)
            .ok_or_else(|| unknown_creature_diagnostic(&creature_id))?;
        mother.health = new_mother_health;

        Ok(noop_result())
    }
}
