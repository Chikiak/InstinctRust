//! Unit tests for the turn interpreter (interp.rs).
//! Split from tests/interp_tests.rs: one file per responsibility.
//! Submodules: actions, arithmetic, common, end_of_program, errors, if_goto,
//! line_limit, runtime_error_bus, turn_flow, wait.

mod actions;
mod arithmetic;
mod common;
mod end_of_program;
mod errors;
mod if_goto;
mod line_limit;
mod runtime_error_bus;
mod turn_flow;
mod wait;

pub use common::{compile_body, run_once, run_turn_full, FakeCtx, SharedOutput};
