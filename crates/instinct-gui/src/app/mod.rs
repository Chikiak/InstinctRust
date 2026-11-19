//! Application state machine without knowledge of eframe/egui.
//! Pure and headless: testable without opening a window.

use crate::render::Cell;

/// State of the GUI application.
#[derive(Default)]
pub struct AppState {
    pub selected: Option<Cell>,
}

/// Messages produced by the UI layer and handled by the state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    CellClicked(Cell),
}

impl AppState {
    /// Pure update of the state from a UI message.
    pub fn update(&mut self, msg: Msg) {
        match msg {
            Msg::CellClicked(cell) => {
                self.selected = Some(cell);
            }
        }
    }
}
