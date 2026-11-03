//! Core of the Instinct interpreter and simulator.
//! No external dependencies; the world emits events, never prints.

pub mod diagnostic;
pub mod expr;
pub mod lexer;
pub mod resources;
pub mod rng;
pub mod value;
pub mod world;
