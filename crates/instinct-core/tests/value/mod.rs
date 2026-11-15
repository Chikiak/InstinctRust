//! Unit tests for the `value` module.
//! Split into submodules: `common` (int/text helpers), `ops` (operators and
//! type errors) and `truthy` (truthiness). Bodies moved verbatim from
//! tests/value_tests.rs; Spanish comments translated to concise English.

mod common;
mod ops;
mod truthy;

pub use common::*;
