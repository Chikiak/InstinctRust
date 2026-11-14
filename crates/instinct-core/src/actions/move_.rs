//! `move(dx, dy, speed)`: advances up to `speed` cells one by one, stopping
//! before the first cell outside the map, with an object, or with a creature.
//! Direction (0, 0) or speed <= 0: does not move and does not fail. dx or dy
//! outside {-1, 0, 1}, or text arguments: error naming the argument.

use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::{CreatureId, World};
use crate::actions::ActionResult;

/// Converts an integer into a grid index (`usize`). Rejects negatives so they
/// never wrap into a huge index, and rejects values that do not fit in
/// `usize`. Defensive in `Move::apply` because `validate_direction` already
/// limits `dx`/`dy` to `{-1, 0, 1}`, but avoids any silent sign conversion.
pub fn index_from_i64(n: i64) -> Option<usize> {
    if n < 0 {
        return None;
    }
    usize::try_from(n).ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move;

impl crate::actions::Action for Move {
    fn name(&self) -> &str {
        "move"
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
        let speed = match &args[2] {
            Value::Int(s) => *s,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "move",
                    1,
                    "argument 'speed' must be an integer, got text".to_string(),
                ));
            }
        };

        // Direction (0, 0) or speed <= 0: no move, no error.
        if (dx == 0 && dy == 0) || speed <= 0 {
            return Ok(ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            });
        }

        // dx or dy outside {-1, 0, 1}: error naming the argument.
        if !((-1..=1).contains(&dx)) {
            return Err(Diagnostic::new(
                "move",
                1,
                format!("argument 'dx' must be -1, 0, or 1, got {dx}"),
            ));
        }
        if !((-1..=1).contains(&dy)) {
            return Err(Diagnostic::new(
                "move",
                1,
                format!("argument 'dy' must be -1, 0, or 1, got {dy}"),
            ));
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

        // Final destination: non-negative coordinates (x/y never become
        // negative due to the previous break, but validated defensively).
        let Some(xi) = index_from_i64(x) else {
            return Err(Diagnostic::new(
                "move",
                1,
                "destination coordinate outside the grid".to_string(),
            ));
        };
        let Some(yi) = index_from_i64(y) else {
            return Err(Diagnostic::new(
                "move",
                1,
                "destination coordinate outside the grid".to_string(),
            ));
        };
        let _ = world.move_creature_to(creature_id, xi, yi);
        Ok(ActionResult {
            sleep_turns: 0,
            events: Vec::new(),
        })
    }
}

/// Parses a move direction argument, returning it as i64 or an error naming
/// the argument if it is text.
pub(super) fn parse_move_arg(value: &Value, arg_name: &str) -> Result<i64, Diagnostic> {
    match value {
        Value::Int(n) => Ok(*n),
        Value::Text(_) => Err(Diagnostic::new(
            "move",
            1,
            format!("argument '{arg_name}' must be an integer, got text"),
        )),
    }
}
