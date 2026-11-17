//! Actions: `wait`, `move`, `say`, `attack`, `consume`, `reproduce`, `roar`.
//!
//! Actions are stateless types; the catalog keeps them as shared references
//! (`Rc<dyn Action>`). The interpreter resolves each name against the catalog
//! validating arity; there are no `if name == ...` chains by name.

pub mod action_catalog;
pub mod action_trait;
pub mod attack;
pub mod catalog;
pub mod combat;
pub mod consts;
pub mod consume;
pub mod context;
pub mod context_helpers;
pub mod events;
pub mod move_;
pub mod parse;
pub mod reproduce;
pub mod roar;
pub mod say;
pub mod shared;
pub mod wait;

pub use action_catalog::ActionCatalog;
pub use action_trait::{Action, ActionResult};
pub use attack::Attack;
pub use catalog::Catalog;
pub use consume::Consume;
pub use context::{ActionContext, ReadContext, WriteContext};
pub use move_::Move;
pub use reproduce::Reproduce;
pub use roar::Roar;
pub use say::Say;
pub use shared::index_from_i64;
pub use wait::Wait;
