//! Integration tests for the `resources` module, split into submodules.
//!
//! One test file per responsibility, mirroring `src/resources/`. Shared
//! imports live in `common` and are re-exported here.
mod common;
mod folder;
mod map;
mod object;
mod resources;
mod terrain;
mod validation;
pub use common::*;
