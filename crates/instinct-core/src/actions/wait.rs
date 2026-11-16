//! `wait(n)`: sleeps `n-1` turns; `n < 1` counts as 1 (zero sleep); text
//! argument fails.

use crate::actions::shared::{action_result, check_arity, parse_int_arg, ACTION_WAIT, ARG_N};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::value::Value;
use crate::world::CreatureId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wait;

impl crate::actions::Action for Wait {
    fn name(&self) -> &str {
        ACTION_WAIT
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
        check_arity(ACTION_WAIT, self.arity(), args.len())?;
        let n = parse_int_arg(&args[0], ACTION_WAIT, ARG_N)?;
        // `n - 1` no puede overflow: `n == i64::MIN` satura a `i64::MIN` y
        // `.max(0)` lo lleva a 0. Se satura también a `u32::MAX` para evitar
        // un cast truncante silencioso cuando `n` excede el rango de `u32`.
        let sleep_turns = n.saturating_sub(1).max(0).min(i64::from(u32::MAX)) as u32;
        Ok(action_result(sleep_turns, Vec::new()))
    }
}
