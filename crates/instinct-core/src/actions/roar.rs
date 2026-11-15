//! `roar(val)`: emits a roar message available the next tick. Any value
//! (integer or text) is valid; an integer is shown as its textual
//! representation. Consumes a turn. Does not damage or kill.

use crate::actions::ActionResult;
use crate::actions::shared::{creature_context, value_to_text};
use crate::diagnostic::Diagnostic;
use crate::event::Event;
use crate::value::Value;
use crate::world::{CreatureId, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roar;

impl crate::actions::Action for Roar {
    fn name(&self) -> &str {
        "roar"
    }
    fn arity(&self) -> usize {
        1
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
        let text = value_to_text(&args[0]);
        let (pos, species, tick) = creature_context(world, creature_id)?;
        world.add_roar(creature_id, species.clone(), pos, text.clone());
        Ok(ActionResult {
            sleep_turns: 0,
            events: vec![Event::Roared {
                tick,
                species,
                pos,
                text,
            }],
        })
    }
}
