//! `see(x, y)`: content of the cell at (x, y).

use crate::functions::arg::arg_as_int;
use crate::functions::Function;
use crate::perceptions::CreatureView;
use crate::value::Value;
use std::rc::Rc;

/// `see(x, y)`: returns -1 (NONE) out of radius, 0 (GROUND) for terrain,
/// 1 (OBJECT) for an object or out-of-map cell, 2 (ALLY) for a same-faction
/// creature (including its own cell), 3 (ENEMY) for an other-faction creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct See;

impl Function for See {
    fn name(&self) -> &str {
        "see"
    }

    fn arity(&self) -> usize {
        2
    }

    fn call(&self, view: &dyn CreatureView, args: &[Value]) -> Result<Value, String> {
        let x = arg_as_int(args, 0, "see", self.arity())?;
        let y = arg_as_int(args, 1, "see", self.arity())?;
        Ok(Value::Int(view.see(x, y)))
    }

    fn clone_box(&self) -> Rc<dyn Function> {
        Rc::new(*self)
    }
}
