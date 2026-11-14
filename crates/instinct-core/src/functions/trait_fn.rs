//! Trait implemented by reserved language functions.

use crate::perceptions::CreatureView;
use crate::value::Value;
use std::rc::Rc;

/// A reserved function of the language. Stateless: no per-creature state.
pub trait Function: std::fmt::Debug {
    /// Function name as it appears in the language (e.g. `see`, `name`).
    fn name(&self) -> &str;

    /// Number of evaluated arguments the function expects.
    fn arity(&self) -> usize;

    /// Call the function with already-evaluated arguments and the creature's
    /// view. Returns a runtime error string on failure (e.g. text argument).
    fn call(&self, view: &dyn CreatureView, args: &[Value]) -> Result<Value, String>;

    /// Clones the function into a reference-counted trait object.
    fn clone_box(&self) -> Rc<dyn Function>;
}
