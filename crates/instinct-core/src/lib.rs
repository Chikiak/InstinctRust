//! Core of the Instinct interpreter and simulator.
//! No external dependencies; the world emits events, never prints.
//!
//! Arquitectura en capas (sección 1.1 de docs/PATRONES_DISENO_ARQUITECTURA.md):
//! dependencia solo hacia abajo, nunca hacia arriba ni circulares:
//!   CLI -> compile/expr/lexer -> interp/program/value
//!         -> world/sim/species/resources/actions/perception
//!         -> diagnostic/event/rng (+ value)
//! Este orden de `pub mod` documenta las capas; no afecta a la compilación.

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
pub mod perception;
pub mod program;
pub mod sim;
pub mod world;
