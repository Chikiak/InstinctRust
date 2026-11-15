//! `wait(n)`: sleeps `n-1` turns; `n < 1` counts as 1 (zero sleep); text
//! argument fails.

use crate::actions::shared::{action_result, parse_int_arg};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

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
        _world: &mut dyn ActionContext,
        _creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let n = parse_int_arg(&args[0], "wait", "n")?;
        let sleep_turns = n.max(1) as u32;
        Ok(action_result(sleep_turns, Vec::new()))
    }
}
