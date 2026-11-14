//! `consume(dx, dy, damage, heal)`: like `attack` on the adjacent cell, and
//! additionally the attacking creature gains `heal` health points (no cap).

use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::{CreatureId, World};
use crate::actions::ActionResult;
use crate::actions::move_::parse_move_arg;
use crate::actions::shared::{heal_attacker, strike_cell, validate_direction};

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
        let dx = parse_move_arg(&args[0], "dx")?;
        let dy = parse_move_arg(&args[1], "dy")?;
        let damage = match &args[2] {
            Value::Int(d) => *d,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "consume",
                    1,
                    "argument 'damage' must be an integer, got text".to_string(),
                ));
            }
        };
        let heal = match &args[3] {
            Value::Int(h) => *h,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "consume",
                    1,
                    "argument 'heal' must be an integer, got text".to_string(),
                ));
            }
        };
        validate_direction("consume", dx, dy)?;

        // Out of map: no-op, neither damage nor healing, turn consumed.
        let pos = world.creature_position(creature_id)?;
        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;
        if tx < 0 || ty < 0 || tx >= world.columns as i64 || ty >= world.rows as i64 {
            return Ok(ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            });
        }

        let events = strike_cell(world, creature_id, dx, dy, damage)?;
        heal_attacker(world, creature_id, heal)?;
        Ok(ActionResult {
            sleep_turns: 0,
            events,
        })
    }
}
