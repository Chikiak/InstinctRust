//! Integration tests for la puerta de determinismo (F5-08 AC2).
//! Headless: sin ventana, sin eframe.

mod common;

use crate::common::{baseline_criaturas, examples_root, DETERMINISMO_SEED, DETERMINISMO_TICKS};
use instinct_core::event::WorldOutput;
use instinct_core::sim::Simulation;
use instinct_core::world::World;
use instinct_gui::app::time::Speed;
use instinct_gui::app::{AppState, Msg};
use std::cell::RefCell;
use std::rc::Rc;

/// Puerto auxiliar para el camino directo (mismo patrón que main.rs y tests/time).
fn direct_port(buffer: Rc<RefCell<instinct_gui::app::log::LogBuffer>>) -> Box<dyn WorldOutput> {
    struct LocalPort(Rc<RefCell<instinct_gui::app::log::LogBuffer>>);
    impl WorldOutput for LocalPort {
        fn emit(&mut self, event: &instinct_core::event::Event) {
            self.0.borrow_mut().push_event(event);
        }
    }
    Box::new(LocalPort(buffer))
}

/// t_determinismo_01: 5 ticks --seed 7 vía flujo GUI (AppState) == 5×
/// Simulation::step_tick directo con seed 7, mismos textos de registro.
#[test]
fn t_determinismo_01_gui_iguala_directo_con_seed7() {
    let session = instinct_gui::app::session::LoadedSession::new(&examples_root());

    // --- Camino A: 5 ticks vía GUI (AppState) con seed 7 ---
    let mut app = AppState {
        session: Some(session),
        ..Default::default()
    };
    // El camino directo compara contra el mapa "Plain with rocks"; la GUI debe
    // preparar sobre el mismo mapa. El índice 0 del catálogo alfabético es
    // "Arena" (9x9), que no tiene las celdas del baseline, por lo que se
    // resuelve el mapa por nombre en lugar de usar el índice 0.
    let prep_map_idx = app
        .session
        .as_ref()
        .unwrap()
        .maps()
        .iter()
        .position(|m| m.name == "Plain with rocks")
        .expect("mapa 'Plain with rocks' debe estar cargado en la sesión");
    app.update(Msg::PrepEnter(prep_map_idx));
    app.prep_state.as_mut().unwrap().recipe.seed = DETERMINISMO_SEED;
    // Colocar las 4 criaturas del baseline.
    for (name, x, y) in baseline_criaturas() {
        let idx = app
            .session
            .as_ref()
            .unwrap()
            .species_names()
            .iter()
            .position(|n| *n == name)
            .unwrap_or_else(|| panic!("especie '{name}' no encontrada"));
        app.update(Msg::PrepPlace(idx, x, y));
    }
    assert_eq!(app.prep_state.as_ref().unwrap().placed_count(), 4);
    // Play arranca la simulación desde la receta (usa recipe.seed = 7).
    app.update(Msg::TickPlay);
    assert!(matches!(
        app.time.state,
        instinct_gui::app::time::RunState::Running(_)
    ));
    assert!(app.sim.is_some());
    // 5 ticks a 2 tps = 2500 ms.
    app.update(Msg::TickAdvance(
        DETERMINISMO_TICKS * Speed::T2.ms_per_tick(),
    ));
    assert_eq!(app.time.tick_count(), DETERMINISMO_TICKS);
    assert_eq!(app.sim.as_ref().unwrap().world().tick(), DETERMINISMO_TICKS);

    let borrow = app.log_buffer.as_ref().unwrap().borrow();
    let entries = borrow.entries().to_vec();
    let gui_events: Vec<String> = entries.into_iter().map(|e| e.line).collect();

    // --- Camino B: 5× Simulation::step_tick directo con seed 7 ---
    let session_b = instinct_gui::app::session::LoadedSession::new(&examples_root());
    let map_b: &instinct_core::resources::Map = session_b
        .maps()
        .iter()
        .find(|m| m.name == "Plain with rocks")
        .expect("map 'Plain with rocks'");
    let mut world = World::from_map(
        map_b,
        &session_b.resources().terrains,
        &session_b.resources().objects,
    );
    for (name, x, y) in baseline_criaturas() {
        let species = session_b
            .species()
            .iter()
            .find(|s| s.name == name)
            .unwrap_or_else(|| panic!("especie '{name}' no encontrada"));
        let species = Rc::new(species.clone());
        world
            .place_creature(&species, x, y)
            .unwrap_or_else(|d| panic!("no se pudo colocar {name}: {d}"));
    }
    let mut sim = Simulation::new(world);
    sim.set_rng_seed(DETERMINISMO_SEED);
    let buffer = Rc::new(RefCell::new(
        instinct_gui::app::log::LogBuffer::with_capacity(1000),
    ));
    sim.world.set_output(direct_port(Rc::clone(&buffer)));
    for _ in 0..DETERMINISMO_TICKS {
        let _ = sim.step_tick();
    }
    let borrow = buffer.borrow();
    let entries = borrow.entries().to_vec();
    let direct_events: Vec<String> = entries.into_iter().map(|e| e.line).collect();

    // Comparar entradas del registro (mismos textos).
    assert_eq!(
        gui_events, direct_events,
        "la GUI no debe perturbar el determinismo:\nGUI: {gui_events:?}\ndirecto: {direct_events:?}"
    );
    // Sanity: debe haber al menos un evento en 5 ticks.
    assert!(
        !gui_events.is_empty(),
        "se esperaban eventos en 5 ticks con seed 7"
    );
}
