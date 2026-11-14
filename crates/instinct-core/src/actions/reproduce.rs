//! `reproduce(dx, dy, health)`: creates an offspring of the same species in
//! the adjacent cell (x+dx, y+dy) of the mother if inside the map and it is
//! only terrain (`CellContent::Empty`) and the mother has health strictly
//! greater than `health`. The mother loses `health`; the offspring is born
//! with health = `health`, age 0, no sleep turns, pc at start (`start:`)
//! and empty variables. Its first turn is the next tick (the `step_tick`
//! snapshot excludes it from the current tick). Any failure is a natural
//! abort: it does nothing, but consumes the turn.

use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::{CreatureId, World};
use crate::actions::ActionResult;
use crate::actions::move_::parse_move_arg;
use crate::actions::shared::validate_direction;

/// No-op action: natural abort. The turn is consumed because
/// `consumes_turn()` is true.
fn abort_reproduction() -> ActionResult {
    ActionResult {
        sleep_turns: 0,
        events: Vec::new(),
    }
}

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
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let dx = parse_move_arg(&args[0], "dx")?;
        let dy = parse_move_arg(&args[1], "dy")?;
        let health = match &args[2] {
            Value::Int(h) => *h,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "reproduce",
                    1,
                    "argument 'health' must be an integer, got text".to_string(),
                ));
            }
        };
        validate_direction("reproduce", dx, dy)?;

        let pos = world.creature_position(creature_id)?;
        let mother = world.creature(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        // The mother must have health strictly greater than `health`.
        if mother.health <= health {
            return Ok(abort_reproduction());
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
        if tx < 0 || ty < 0 || tx >= world.columns as i64 || ty >= world.rows as i64 {
            return Ok(abort_reproduction());
        }
        // `place_creature` validates bounds and free cell: occupied cell ->
        // natural abort without touching the mother.
        let child_id = match world.place_creature(&species, tx as usize, ty as usize) {
            Ok(id) => id,
            Err(_) => return Ok(abort_reproduction()),
        };
        // `place_creature` initializes with `species.health`; adjust to `health`.
        world.set_creature_health(child_id, health)?;

        let mother = world.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        mother.health = new_mother_health;

        Ok(abort_reproduction())
    }
}
