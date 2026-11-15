//! `consume(dx, dy, damage, heal)`: like `attack` on the adjacent cell, and
//! additionally the attacking creature gains `heal` health points (no cap).

use crate::actions::shared::{
    heal_attacker, is_out_of_bounds, out_of_map_noop, parse_dir_arg, parse_int_arg, strike_cell,
};
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::{CreatureId, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consume;

impl crate::actions::Action for Consume {
    fn name(&self) -> &str {
        "consume"
    }
    fn arity(&self) -> usize {
        4
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
        let dx = parse_dir_arg(&args[0], "consume", "dx")?;
        let dy = parse_dir_arg(&args[1], "consume", "dy")?;
        let damage = parse_int_arg(&args[2], "consume", "damage")?;
        let heal = parse_int_arg(&args[3], "consume", "heal")?;

        // Out of map: no-op, neither damage nor healing, turn consumed.
        let pos = world.creature_position(creature_id)?;
        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;
        if is_out_of_bounds(world, tx, ty) {
            return Ok(out_of_map_noop());
        }

        let events = strike_cell(world, creature_id, dx, dy, damage)?;
        heal_attacker(world, creature_id, heal)?;
        Ok(ActionResult {
            sleep_turns: 0,
            events,
        })
    }
}
