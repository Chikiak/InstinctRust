//! Reserved functions: `see(x, y)` and `name(x, y)`.
//!
//! Functions are stateless types resolved against a catalog by name, with
//! arity validated at compile time. Resolution is purely catalog-based — no
//! `if name == ...` chains by name — mirroring `Action`/`Catalog`.

pub mod arg;
pub mod catalog;
pub mod name;
pub mod see;
pub mod trait_fn;

pub use catalog::FunctionCatalog;
pub use name::Name;
pub use see::See;
pub use trait_fn::Function;
