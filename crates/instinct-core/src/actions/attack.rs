//! `attack(dx, dy, damage)`: strikes the adjacent cell (x+dx, y+dy) of the
//! attacking creature. Reduces a creature's health, an object's reserve, or
//! the terrain reserve of that cell. dx/dy outside {-1, 0, 1} or text
//! arguments fail naming the argument. Destination out of map: no-op, turn
//! consumed, no error. A creature dropping to health <= 0 dies on the spot
//! (cell freed, Died cause "health") in the same tick.

use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::actions::shared::{parse_dir_arg, parse_int_arg, strike_cell};
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attack;

impl crate::actions::Action for Attack {
    fn name(&self) -> &str {
        "attack"
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
        let dx = parse_dir_arg(&args[0], "attack", "dx")?;
        let dy = parse_dir_arg(&args[1], "attack", "dy")?;
        let damage = parse_int_arg(&args[2], "attack", "damage")?;

        let events = strike_cell(world, creature_id, dx, dy, damage)?;
        Ok(ActionResult {
            sleep_turns: 0,
            events,
        })
    }
}
