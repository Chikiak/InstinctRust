//! `attack(dx, dy, damage)`: strikes the adjacent cell (x+dx, y+dy) of the
//! attacking creature. Reduces a creature's health, an object's reserve, or
//! the terrain reserve of that cell. dx/dy outside {-1, 0, 1} or text
//! arguments fail naming the argument. Destination out of map: no-op, turn
//! consumed, no error. A creature dropping to health <= 0 dies on the spot
//! (cell freed, Died cause "vida") in the same tick.

use crate::actions::move_::parse_move_arg;
use crate::actions::shared::{strike_cell, validate_direction};
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::{CreatureId, World};

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
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let dx = parse_move_arg(&args[0], "dx")?;
        let dy = parse_move_arg(&args[1], "dy")?;
        let damage = match &args[2] {
            Value::Int(d) => *d,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "attack",
                    1,
                    "argument 'damage' must be an integer, got text".to_string(),
                ));
            }
        };
        validate_direction("attack", dx, dy)?;

        let events = strike_cell(world, creature_id, dx, dy, damage)?;
        Ok(ActionResult {
            sleep_turns: 0,
            events,
        })
    }
}
