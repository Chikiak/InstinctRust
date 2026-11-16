//! `attack(dx, dy, damage)`: strikes the adjacent cell (x+dx, y+dy) of the
//! attacking creature. Reduces a creature's health, an object's reserve, or
//! the terrain reserve of that cell. dx/dy outside {-1, 0, 1} or text
//! arguments fail naming the argument. Destination out of map: no-op, turn
//! consumed, no error. A creature dropping to health <= 0 dies on the spot
//! (cell freed, Died cause "health") in the same tick.

use crate::actions::shared::{apply_strike, check_arity, parse_strike_args, ACTION_ATTACK};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attack;

impl crate::actions::Action for Attack {
    fn name(&self) -> &str {
        ACTION_ATTACK
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
        check_arity(ACTION_ATTACK, self.arity(), args.len())?;
        let (dx, dy, damage) = parse_strike_args(ACTION_ATTACK, args)?;

        apply_strike(world, creature_id, dx, dy, damage, None)
    }
}
