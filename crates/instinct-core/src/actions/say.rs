//! `say(val)`: emits a Said event with the creature's current position and
//! tick, does not consume a turn. A number is shown as text.

use crate::diagnostic::Diagnostic;
use crate::event::Event;
use crate::value::Value;
use crate::world::{CreatureId, World};
use crate::actions::ActionResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Say;

impl crate::actions::Action for Say {
    fn name(&self) -> &str {
        "say"
    }
    fn arity(&self) -> usize {
        1
    }
    fn consumes_turn(&self) -> bool {
        false
    }
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let text = match &args[0] {
            Value::Int(n) => n.to_string(),
            Value::Text(s) => s.clone(),
        };
        let pos = world.creature_position(creature_id)?;
        let species = world.creature_species(creature_id)?;
        let tick = world.tick();
        Ok(ActionResult {
            sleep_turns: 0,
            events: vec![Event::Said {
                tick,
                species,
                pos,
                text,
            }],
        })
    }
}
