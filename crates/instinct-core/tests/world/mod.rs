//! Unit tests for the `world` module.
//! Split into submodules: common (shared helpers and re-exports), draw_errors,
//! map, memento, place, render, reserve, robustness, species, vision.
//! No test logic changed: only code moved from tests/world_tests.rs and
//! Spanish comments translated to concise English.

mod common;
mod draw_errors;
mod map;
mod memento;
mod place;
mod render;
mod reserve;
mod robustness;
mod snapshot;
mod species;
mod vision;

pub use common::*;
