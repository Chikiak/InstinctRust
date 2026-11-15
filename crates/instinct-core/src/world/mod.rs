//! World model: terrain grid, objects, creatures, ticks, and events.
//!
//! The world owns the seeded generator (with interior mutability confined to
//! the generator) and the shared catalog of actions and perceptions. Each
//! creature keeps its own variable map.

pub mod action_context;
pub mod clone_debug;
pub mod context;
pub mod creature;
pub mod draw;
pub mod grid;
pub mod laws;
pub mod life;
pub mod roars;
pub mod snapshot;
pub mod ticks;
#[allow(clippy::module_inception)]
pub mod world;

pub use context::CreatureContext;
pub use creature::{CellContent, Creature, CreatureId, CreatureMemento, CreatureState};
pub use roars::Roar;
pub use world::World;
