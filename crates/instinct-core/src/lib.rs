//! Core of the Instinct interpreter and simulator.
//! No external dependencies; the world emits events, never prints.

pub mod actions;
pub mod compile;
pub mod diagnostic;
pub mod event;
pub mod expr;
pub mod functions;
pub mod interp;
pub mod lexer;
pub mod perception;
pub mod program;
pub mod resources;
pub mod rng;
pub mod sim;
pub mod species;
pub mod value;
pub mod world;
