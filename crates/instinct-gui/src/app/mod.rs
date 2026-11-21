//! Application state machine without knowledge of eframe/egui.
//! Pure and headless: testable without opening a window.

pub mod log;
pub mod prep;
pub mod session;
pub mod time;

use crate::app::log::{LogBuffer, LogEntry, LogEntryKind};
use crate::app::prep::PlacementId;
use crate::app::session::LoadedSession;
use crate::app::time::{RunState, Speed, TimeControl};
use crate::render::Cell;
use instinct_core::event::WorldOutput;
use instinct_core::sim::Simulation;
use std::cell::RefCell;
use std::rc::Rc;

/// Port that forwards world events to a shared `LogBuffer`.
struct LogPort(Rc<RefCell<LogBuffer>>);

impl WorldOutput for LogPort {
    fn emit(&mut self, event: &instinct_core::event::Event) {
        self.0.borrow_mut().push_event(event);
    }
}

/// State of the GUI application.
#[derive(Default)]
pub struct AppState {
    pub selected: Option<Cell>,
    pub session: Option<LoadedSession>,
    pub prep_state: Option<prep::PrepState>,
    pub prep_selected_species: usize,
    pub time: TimeControl,
    pub sim: Option<Simulation>,
    pub log_buffer: Option<Rc<RefCell<LogBuffer>>>,
    /// Eventos del último `step_tick` (se vacían en el núcleo tras cada tick;
    /// se conservan aquí para que la UI pueda leer `Event::Roared` sin
    /// depender de `snapshot.events`, que está vacío tras `drain_events`).
    pub last_events: Vec<instinct_core::event::Event>,
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
                // Rechazar colocación si la simulación está corriendo o en pausa.
                if matches!(self.time.state, RunState::Running(_) | RunState::Paused(_)) {
                    return;
                }
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
                // Rechazar remoción si la simulación está corriendo o en pausa.
                if matches!(self.time.state, RunState::Running(_) | RunState::Paused(_)) {
                    return;
                }
                if let Some(prep_state) = self.prep_state.as_mut() {
                    let _ = prep_state.remove(placement_id);
                }
            }
            Msg::PrepSelectSpecies(idx) => {
                self.prep_selected_species = idx;
            }
            Msg::TickPlay => {
                // Sin preparación no hay mundo que simular.
                if self.prep_state.is_none() {
                    return;
                }
                self.time.play();
                if matches!(self.time.state, RunState::Running(_))
                    && self.sim.is_none()
                    && self.prep_state.is_some()
                {
                    self.start_simulation();
                }
            }
            Msg::TickPause => {
                self.time.pause();
            }
            Msg::TickStep => {
                if self.sim.is_none() {
                    self.start_simulation();
                }
                if let Some(sim) = self.sim.as_mut() {
                    self.time.step_once();
                    self.last_events = sim.step_tick();
                }
            }
            Msg::TickSetSpeed(speed) => {
                self.time.set_speed(speed);
            }
            Msg::TickAdvance(elapsed_ms) => {
                let n = self.time.advance(elapsed_ms);
                if let Some(sim) = self.sim.as_mut() {
                    for _ in 0..n {
                        self.last_events = sim.step_tick();
                    }
                }
            }
            Msg::Restart => {
                self.restart();
            }
        }
    }

    /// Reinicia la simulación al estado de preparación: recrea el mundo desde la
    /// receta guardada en `prep_state` (mismo mapa, mismas colocaciones, misma
    /// semilla vía `set_rng_seed`), nunca `World::clone`. Queda en preparación
    /// con `sim = None` y empuja una entrada de registro indicando el reinicio.
    pub fn restart(&mut self) {
        let prep = match self.prep_state.as_ref() {
            Some(prep) => prep,
            None => return,
        };
        let seed = prep.recipe.seed;
        let map_name = prep.recipe.map_name.clone();
        let placements = prep.recipe.placements.clone();

        // Recrear la preparación desde la receta: from_map + place por cada
        // placement + set_rng_seed(seed). Nunca World::clone.
        match prep::PrepState::enter_prep(
            self.session
                .as_ref()
                .expect("sesión cargada para reiniciar"),
            &map_name,
            seed,
        ) {
            Ok(mut new_prep) => {
                // Re-aplicar las colocaciones de la receta (mismo orden).
                let session = self
                    .session
                    .as_ref()
                    .expect("sesión cargada para reiniciar");
                for (species_name, x, y) in placements {
                    let _ = new_prep.place(session, &species_name, x, y);
                }
                self.prep_state = Some(new_prep);
            }
            Err(_diagnostic) => {
                // Mantener la prep_state actual si falla la recreación.
                return;
            }
        }

        // Limpiar simulación y eventos del último tick.
        self.sim = None;
        self.last_events.clear();

        // Pausar el tiempo y resetear su contador de ticks (estado de preparación).
        self.time.state = crate::app::time::RunState::Idle;
        self.time.tick = 0;
        self.time.accumulator_ms = 0;

        // Entrada de registro indicando el reinicio.
        if let Some(buffer) = self.log_buffer.as_ref() {
            buffer.borrow_mut().push(LogEntry {
                tick: 0,
                kind: LogEntryKind::Event,
                line: "[reinicio] mundo recreado desde la receta; queda en preparación".to_string(),
            });
        }
    }

    /// Construye la `Simulation` a partir del `prep_state` (receta: mapa +
    /// colocaciones + semilla), conecta el puerto de salida al `log_buffer` y
    /// siembra el RNG. Invariante: `time.tick == sim.world().tick()`.
    fn start_simulation(&mut self) {
        let Some(prep) = self.prep_state.as_ref() else {
            return;
        };
        let Some(session) = self.session.as_ref() else {
            return;
        };

        // Reconstruir el mundo desde la receta: from_map + place_creature +
        // set_rng_seed(seed). Nunca World::clone.
        let map = session
            .maps()
            .iter()
            .find(|m| m.name == prep.recipe.map_name)
            .expect("mapa de la receta debe existir en la sesión");

        let mut world = instinct_core::world::World::from_map(
            map,
            &session.resources().terrains,
            &session.resources().objects,
        );

        for (species_name, x, y) in &prep.recipe.placements {
            let species = session
                .species()
                .iter()
                .find(|s| s.name == *species_name)
                .expect("especie de la receta debe existir en la sesión");
            let species = Rc::new(species.clone());
            let _ = world.place_creature(&species, *x, *y);
        }

        let mut sim = Simulation::new(world);
        sim.set_rng_seed(prep.recipe.seed);

        // Conectar el puerto de salida al log_buffer compartido.
        let buffer = Rc::new(RefCell::new(LogBuffer::with_capacity(4096)));
        sim.world.set_output(Box::new(LogPort(Rc::clone(&buffer))));
        self.log_buffer = Some(buffer);
        self.sim = Some(sim);
    }
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
    /// Play the simulation.
    TickPlay,
    /// Pause the simulation.
    TickPause,
    /// Step exactly one tick.
    TickStep,
    /// Set the simulation speed.
    TickSetSpeed(Speed),
    /// Advance the simulation by `elapsed_ms` (accumulator-based ticks).
    TickAdvance(u64),
    /// Reinicia la simulación al estado de preparación (desde la receta).
    Restart,
}
