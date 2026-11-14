//! Integration tests for the `actions` module, split into submodules.
//!
//! Each action type has its own test file. Shared helpers live in `common`.

mod attack;
mod catalog;
mod common;
mod consume;
mod index;
mod move_;
mod reproduce;
mod roar;
mod say;
mod wait;

pub use common::*;
