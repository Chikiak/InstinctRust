//! Actions: `wait`, `move`, `say`, `attack`, `consume`, `reproduce`, `roar`.
//!
//! Actions are stateless types; the catalog keeps them as shared references
//! (`Rc<dyn Action>`). The interpreter resolves each name against the catalog
//! validating arity; there are no `if name == ...` chains by name.

pub mod action_catalog;
pub mod attack;
pub mod catalog;
pub mod context;
pub mod consume;
pub mod move_;
pub mod reproduce;
pub mod roar;
pub mod say;
pub mod shared;
pub mod r#trait;
pub mod wait;

pub use action_catalog::ActionCatalog;
pub use attack::Attack;
pub use catalog::Catalog;
pub use consume::Consume;
pub use context::ActionContext;
pub use move_::{index_from_i64, Move};
pub use r#trait::{Action, ActionResult};
pub use reproduce::Reproduce;
pub use roar::Roar;
pub use say::Say;
pub use wait::Wait;
