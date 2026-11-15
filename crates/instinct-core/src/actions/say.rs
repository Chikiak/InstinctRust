//! `say(val)`: emits a Said event with the creature's current position and
//! tick, does not consume a turn. A number is shown as text.

use crate::actions::shared::{action_result, creature_context, value_to_text, ACTION_SAY};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::event::Event;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Say;

impl crate::actions::Action for Say {
    fn name(&self) -> &str {
        ACTION_SAY
    }
    fn arity(&self) -> usize {
        1
    }
    fn consumes_turn(&self) -> bool {
        false
    }
    fn apply(
        &self,
        world: &mut dyn ActionContext,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let text = value_to_text(&args[0]);
        let (pos, species, tick) = creature_context(world, creature_id)?;
        Ok(action_result(
            0,
            vec![Event::Said {
                tick,
                species,
                pos,
                text,
            }],
        ))
    }
}
