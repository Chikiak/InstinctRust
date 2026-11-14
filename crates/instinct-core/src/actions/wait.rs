//! `wait(n)`: sleeps `n-1` turns; `n < 1` counts as 1 (zero sleep); text
//! argument fails.

use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::{CreatureId, World};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wait;

impl crate::actions::Action for Wait {
    fn name(&self) -> &str {
        "wait"
    }
    fn arity(&self) -> usize {
        1
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        _world: &mut World,
        _creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let n = match &args[0] {
            Value::Int(n) => *n,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "wait",
                    1,
                    "argument 'n' must be an integer, got text".to_string(),
                ));
            }
        };
        let sleep_turns = n.max(1) as u32;
        Ok(ActionResult {
            sleep_turns,
            events: Vec::new(),
        })
    }
}
