//! Unit tests for `.ins` compilation (F0-06): header, body, labels, names,
//! arity, and diagnostics.
//! Split from `tests/compile_tests.rs` into submodules: one file per
//! responsibility. No test logic changed: only code moved from
//! `tests/compile_tests.rs` and Spanish comments translated to concise
//! English.
//!
//! Submodules: common (shared helpers and re-exports), walker, header, body,
//! labels, names, diagnostics.

mod common;
mod walker;
mod header;
mod body;
mod labels;
mod names;
mod diagnostics;

pub use common::*;
