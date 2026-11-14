//! `roar(val)`: emits a roar message available the next tick. Any value
//! (integer or text) is valid; an integer is shown as its textual
//! representation. Consumes a turn. Does not damage or kill.

use crate::diagnostic::Diagnostic;
use crate::event::Event;
use crate::value::Value;
use crate::world::{CreatureId, World};
use crate::actions::ActionResult;

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
        let text = match &args[0] {
            Value::Int(n) => n.to_string(),
            Value::Text(s) => s.clone(),
        };
        let pos = world.creature_position(creature_id)?;
        let species = world.creature_species(creature_id)?;
        let tick = world.tick();
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
