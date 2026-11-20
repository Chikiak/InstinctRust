//! Graphical adapter of the Instinct simulator.

pub mod app;
pub mod render;
pub mod ui;

pub use app::log::{LogBuffer, LogEntry};
pub use app::session::LoadedSession;
pub use app::{AppState, Msg};
pub use render::{cell_to_pixel, pixel_to_cell, Cell, GRID_COLS, GRID_ROWS};
