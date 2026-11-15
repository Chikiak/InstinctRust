//! `move(dx, dy, speed)`: advances up to `speed` cells one by one, stopping
//! before the first cell outside the map, with an object, or with a creature.
//! Direction (0, 0) or speed <= 0: does not move and does not fail. dx or dy
//! outside {-1, 0, 1}, or text arguments: error naming the argument.

use crate::actions::shared::{
    index_from_i64, out_of_map_noop, parse_dir_args, parse_int_arg, ACTION_MOVE, ARG_SPEED,
};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move;

impl crate::actions::Action for Move {
    fn name(&self) -> &str {
        ACTION_MOVE
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
        let (dx, dy) = parse_dir_args(args, ACTION_MOVE)?;
        let speed = parse_int_arg(&args[2], ACTION_MOVE, ARG_SPEED)?;

        // Direction (0, 0) or speed <= 0: no move, no error.
        if (dx == 0 && dy == 0) || speed <= 0 {
            return Ok(out_of_map_noop());
        }

        let pos = world.creature_position(creature_id)?;
        let mut x = pos.x as i64;
        let mut y = pos.y as i64;

        for _ in 0..speed {
            let nx = x + dx;
            let ny = y + dy;
            // Rejects negative coordinates: they are never converted to usize.
            let Some(nxi) = index_from_i64(nx) else { break };
            let Some(nyi) = index_from_i64(ny) else { break };
            if !world.cell_is_free(nxi, nyi) {
                break;
            }
            x = nx;
            y = ny;
        }

        world.move_creature_to(creature_id, x as usize, y as usize)?;
        Ok(out_of_map_noop())
    }
}
