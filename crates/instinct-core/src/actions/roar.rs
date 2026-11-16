//! `roar(val)`: emits a roar message available the next tick. Any value
//! (integer or text) is valid; an integer is shown as its textual
//! representation. Consumes a turn. Does not damage or kill.

use crate::actions::shared::{
    check_arity, creature_context, say_like, value_to_text, ACTION_ROAR,
};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roar;

impl crate::actions::Action for Roar {
    fn name(&self) -> &str {
        ACTION_ROAR
    }
    fn arity(&self) -> usize {
        1
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
        check_arity(ACTION_ROAR, self.arity(), args.len())?;
        let text = value_to_text(&args[0]);
        let (pos, species, tick) = creature_context(world, creature_id)?;
        say_like(world, creature_id, species, pos, tick, text, true)
    }
}
