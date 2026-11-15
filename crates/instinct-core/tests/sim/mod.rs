//! Simulation scenario tests, split from `tests/sim_tests.rs`.
//! Submodules: death, determinism, memento, perceptions, reproduce, roars,
//! terrain, text_semantics. Shared helpers live in `common`.

pub mod common;
pub mod death;
pub mod determinism;
pub mod memento;
pub mod perceptions;
pub mod reproduce;
pub mod roars;
pub mod terrain;
pub mod text_semantics;

pub use common::{
    build_world, empty_map, grass_terrain, rock_object, species_no_program, species_with_program,
    species_with_program_and_faction, TestRng,
};
