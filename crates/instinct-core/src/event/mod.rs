//! Events emitted by the world during simulation.
//! The world never prints; it only emits events that consumers (CLI, GUI)
//! render.

pub mod display;
pub mod events;
pub mod output;

pub use events::{Event, Pos};
pub use output::WorldOutput;
