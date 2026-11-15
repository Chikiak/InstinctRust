//! `reproduce(dx, dy, health)`: creates an offspring of the same species in
//! the adjacent cell (x+dx, y+dy) of the mother if inside the map and it is
//! only terrain (`CellContent::Empty`) and the mother has health strictly
//! greater than `health`. The mother loses `health`; the offspring is born
//! with health = `health`, age 0, no sleep turns, pc at start (`start:`)
//! and empty variables. Its first turn is the next tick (the `step_tick`
//! snapshot excludes it from the current tick). Any failure is a natural
//! abort: it does nothing, but consumes the turn.

use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::actions::shared::{
    is_out_of_bounds, natural_abort_noop, out_of_map_noop, parse_dir_arg, parse_int_arg,
};
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reproduce;

impl crate::actions::Action for Reproduce {
    fn name(&self) -> &str {
        "reproduce"
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
        let dx = parse_dir_arg(&args[0], "reproduce", "dx")?;
        let dy = parse_dir_arg(&args[1], "reproduce", "dy")?;
        let health = parse_int_arg(&args[2], "reproduce", "health")?;

        let pos = world.creature_position(creature_id)?;
        let mother = world.creature(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        // The mother must have health strictly greater than `health`.
        if mother.health <= health {
            // Aborto natural por salud insuficiente de la madre.
            return Ok(natural_abort_noop());
        }
        let species = mother.species.clone();
        let mother_health = mother.health;

        // New mother health with checked arithmetic, before mutating anything.
        let new_mother_health = mother_health.checked_sub(health).ok_or_else(|| {
            Diagnostic::new(
                "reproduce",
                1,
                format!("overflow reducing mother health at ({},{})", pos.x, pos.y),
            )
        })?;

        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;
        // Out of map: natural abort.
        if is_out_of_bounds(world, tx, ty) {
            return Ok(out_of_map_noop());
        }
        // `place_creature` validates bounds and free cell: occupied cell ->
        // natural abort without touching the mother.
        let child_id = match world.place_creature(&species, tx as usize, ty as usize) {
            Ok(id) => id,
            Err(_) => return Ok(out_of_map_noop()),
        };
        // `place_creature` initializes with `species.health`; adjust to `health`.
        world.set_creature_health(child_id, health)?;

        let mother = world.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        mother.health = new_mother_health;

        Ok(out_of_map_noop())
    }
}
