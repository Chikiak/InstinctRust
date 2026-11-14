//! Integration tests for perceptions, split into submodules.
//!
//! Each responsibility group has its own test file. Shared helpers live in
//! `common`.

mod catalog;
mod common;
mod creature_state;
mod random;
mod self_world;
mod variables;
mod nearby;

pub use common::*;
