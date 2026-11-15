//! Unit tests for expression parsing and evaluation.
//!
//! Split into submodules: `precedence`, `logic`, `syntax`, `calls`, `visitor`.
//! Shared helpers live in `helpers`.

mod calls;
mod helpers;
mod logic;
mod precedence;
mod syntax;
mod visitor;

pub use helpers::*;
