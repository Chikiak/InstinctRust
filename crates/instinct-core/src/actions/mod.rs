//! Actions: `wait`, `move`, `say`, `attack`, `consume`, `reproduce`, `roar`.
//!
//! Actions are stateless types; the catalog keeps them as shared references
//! (`Rc<dyn Action>`). The interpreter resolves each name against the catalog
//! validating arity; there are no `if name == ...` chains by name.

pub mod attack;
pub mod catalog;
pub mod consume;
pub mod move_;
pub mod r#trait;
pub mod reproduce;
pub mod roar;
pub mod say;
pub mod shared;
pub mod wait;

pub use attack::Attack;
pub use catalog::Catalog;
pub use consume::Consume;
pub use move_::{index_from_i64, Move};
pub use r#trait::{Action, ActionResult};
pub use reproduce::Reproduce;
pub use roar::Roar;
pub use say::Say;
pub use wait::Wait;
