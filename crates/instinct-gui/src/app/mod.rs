//! Application state machine without knowledge of eframe/egui.
//! Pure and headless: testable without opening a window.

pub mod log;
pub mod prep;
pub mod session;

use crate::app::prep::PlacementId;
use crate::app::session::LoadedSession;
use crate::render::Cell;

/// State of the GUI application.
#[derive(Default)]
pub struct AppState {
    pub selected: Option<Cell>,
    pub session: Option<LoadedSession>,
    pub prep_state: Option<prep::PrepState>,
    pub prep_selected_species: usize,
}

/// Messages produced by the UI layer and handled by the state machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    CellClicked(Cell),
    MapSelected(usize),
    SpeciesSelected(usize),
    /// Enter preparation for the map at the given index in the session catalog.
    PrepEnter(usize),
    /// Place the selected species at grid cell (x, y).
    PrepPlace(usize, usize, usize),
    /// Remove the placement with the given id.
    PrepRemove(PlacementId),
    /// Select the species at the given index for placement.
    PrepSelectSpecies(usize),
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
            Msg::PrepEnter(map_idx) => {
                let Some(session) = self.session.as_mut() else {
                    return;
                };
                let Some(map) = session.maps().get(map_idx) else {
                    return;
                };
                let seed = 0;
                match prep::PrepState::enter_prep(session, &map.name, seed) {
                    Ok(prep_state) => {
                        self.prep_state = Some(prep_state);
                        self.prep_selected_species = 0;
                    }
                    Err(_diagnostic) => {
                        // TODO: surface diagnostic in the log.
                    }
                }
            }
            Msg::PrepPlace(species_idx, x, y) => {
                let Some(prep_state) = self.prep_state.as_mut() else {
                    return;
                };
                let Some(session) = self.session.as_ref() else {
                    return;
                };
                let Some(species) = session.species().get(species_idx) else {
                    return;
                };
                let _ = prep_state.place(session, &species.name, x, y);
            }
            Msg::PrepRemove(placement_id) => {
                if let Some(prep_state) = self.prep_state.as_mut() {
                    let _ = prep_state.remove(placement_id);
                }
            }
            Msg::PrepSelectSpecies(idx) => {
                self.prep_selected_species = idx;
            }
        }
    }
}
