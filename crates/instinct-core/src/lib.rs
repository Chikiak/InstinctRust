//! Core of the Instinct interpreter and simulator.
//! No external dependencies; the world emits events, never prints.
//!
//! Layered architecture (section 1.1 of docs/PATRONES_DISENO_ARQUITECTURA.md):
//! dependencies only downward, never upward or circular:
//!   CLI -> compile/expr/lexer -> interp/program/value
//!         -> world/sim/species/resources/actions/perception
//!         -> diagnostic/event/rng (+ value)
//! This `pub mod` order documents the layers; does not affect compilation.

pub mod diagnostic;
pub mod event;
pub mod rng;
pub mod value;

pub mod functions;
pub mod resources;
pub mod species;

pub mod actions;
pub mod compile;
pub mod expr;
pub mod interp;
pub mod lexer;
pub mod perceptions;
pub mod program;
pub mod sim;
pub mod world;
