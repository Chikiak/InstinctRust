//! `consume(dx, dy, damage, heal)`: like `attack` on the adjacent cell, and
//! additionally the attacking creature gains `heal` health points (no cap).

use crate::actions::shared::{
    action_result, heal_attacker, out_of_map_noop, parse_dir_args, parse_int_arg, strike_cell,
    ACTION_CONSUME, ARG_DAMAGE, ARG_HEAL,
};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consume;

impl crate::actions::Action for Consume {
    fn name(&self) -> &str {
        ACTION_CONSUME
    }
    fn arity(&self) -> usize {
        4
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
        let (dx, dy) = parse_dir_args(args, ACTION_CONSUME)?;
        let damage = parse_int_arg(&args[2], ACTION_CONSUME, ARG_DAMAGE)?;
        let heal = parse_int_arg(&args[3], ACTION_CONSUME, ARG_HEAL)?;

        let events = strike_cell(world, creature_id, dx, dy, damage)?;

        // Si strike_cell devolvió eventos vacíos, el objetivo estaba fuera del mapa.
        // strike_cell solo devuelve eventos vacíos en caso de OOB; en cualquier otro
        // caso genera al menos un damaged_event. No curar en ese caso.
        if events.is_empty() {
            return Ok(out_of_map_noop());
        }

        heal_attacker(world, creature_id, heal)?;
        Ok(action_result(0, events))
    }
}
