//! Unit tests for the `event` module.
//! Split into submodules: `common` (event fields/equality), `shared`
//! (SharedOutput test harness), `display` (Event Display) and
//! `world_output` (WorldOutput wiring). Bodies moved verbatim from
//! tests/event_tests.rs; Spanish comments translated to concise English.

mod common;
mod display;
mod shared;
mod world_output;
