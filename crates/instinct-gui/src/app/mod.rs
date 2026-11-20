//! Application state machine without knowledge of eframe/egui.
//! Pure and headless: testable without opening a window.

pub mod log;
pub mod session;

use crate::app::session::LoadedSession;
use crate::render::Cell;

/// State of the GUI application.
#[derive(Default)]
pub struct AppState {
    pub selected: Option<Cell>,
    pub session: Option<LoadedSession>,
}

/// Messages produced by the UI layer and handled by the state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    CellClicked(Cell),
    MapSelected(usize),
    SpeciesSelected(usize),
}

impl AppState {
    /// Pure update of the state from a UI message.
    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::CellClicked(cell) => {
                self.selected = Some(cell);
            }
            Msg::MapSelected(idx) => {
                if let Some(session) = self.session.as_mut() {
                    session.select_map(idx);
                }
            }
            Msg::SpeciesSelected(idx) => {
                if let Some(session) = self.session.as_mut() {
                    session.select_species(idx);
                }
            }
        }
    }
}
