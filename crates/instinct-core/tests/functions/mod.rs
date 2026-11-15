//! Integration tests for `see`/`name` functions and constants (F3-03).
//! Uses the real world, the real catalog, and the real evaluation context.
//!
//! Split into submodules: `see`, `name`, `constants`, `creature`. Shared helpers live in `common`.

mod common;
mod constants;
mod creature;
mod name;
mod see;

pub use common::*;
