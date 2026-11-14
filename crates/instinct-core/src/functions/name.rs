//! `name(x, y)`: text name of the element at (x, y).

use crate::functions::arg::arg_as_int;
use crate::functions::Function;
use crate::perceptions::CreatureView;
use crate::value::Value;
use std::rc::Rc;

/// `name(x, y)`: returns the species name for a creature, the object name
/// for an object, the terrain name otherwise, and empty text out of radius
/// or out of map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Name;

impl Function for Name {
    fn name(&self) -> &str {
        "name"
    }

    fn arity(&self) -> usize {
        2
    }

    fn call(&self, view: &dyn CreatureView, args: &[Value]) -> Result<Value, String> {
        let x = arg_as_int(args, 0, "name", self.arity())?;
        let y = arg_as_int(args, 1, "name", self.arity())?;
        Ok(Value::Text(view.name(x, y).to_string()))
    }

    fn clone_box(&self) -> Rc<dyn Function> {
        Rc::new(*self)
    }
}
