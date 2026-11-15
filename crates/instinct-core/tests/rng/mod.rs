//! Unit tests for the `rng` module.
//! Split into submodules: `common` (TestRng helper), `gen` (RNG generation
//! tests) and `world` (World RNG source wiring). Bodies moved verbatim from
//! tests/rng_tests.rs; comments translated to concise English.

mod common;
mod gen;
mod world;

pub use common::*;
