//! Integration tests for `app::time`: play, pausa, paso y velocidades 2/10/30 tps.
//! Headless: sin ventana, sin eframe.

mod common;

use crate::common::*;
use instinct_core::event::WorldOutput;
use instinct_core::sim::Simulation;
use instinct_core::world::World;
use instinct_gui::app::prep::PlacementId;
use instinct_gui::app::time::{RunState, Speed, TimeControl};
use instinct_gui::app::{AppState, Msg};
use std::cell::RefCell;
use std::rc::Rc;

/// t-time-01: `Speed::tps` devuelve exactamente 2 / 10 / 30.
#[test]
fn t_time_01_speed_tps_exact() {
    assert_eq!(Speed::T2.tps(), 2);
    assert_eq!(Speed::T10.tps(), 10);
    assert_eq!(Speed::T30.tps(), 30);
}

/// t-time-02: `Speed::ms_per_tick` devuelve 500 / 100 / 33.
#[test]
fn t_time_02_ms_per_tick() {
    assert_eq!(Speed::T2.ms_per_tick(), 500);
    assert_eq!(Speed::T10.ms_per_tick(), 100);
    assert_eq!(Speed::T30.ms_per_tick(), 33);
}

/// t-time-03: `advance` a 2 tps: 500 ms produce 1 tick, 1000 ms produce 2.
#[test]
fn t_time_03_advance_2_tps() {
    let mut tc = TimeControl::new();
    tc.play();
    assert_eq!(tc.advance(500), 1);
    assert_eq!(tc.tick_count(), 1);
    assert_eq!(tc.advance(500), 1);
    assert_eq!(tc.tick_count(), 2);
}

/// t-time-04: `advance` a 10 tps: 100 ms produce 1 tick.
#[test]
fn t_time_04_advance_10_tps() {
    let mut tc = TimeControl::new();
    tc.play();
    tc.set_speed(Speed::T10);
    assert_eq!(tc.advance(100), 1);
    assert_eq!(tc.tick_count(), 1);
    assert_eq!(tc.advance(200), 2);
    assert_eq!(tc.tick_count(), 3);
}

/// t-time-05: `advance` a 30 tps: 33 ms produce 1 tick, residuo se conserva.
#[test]
fn t_time_05_advance_30_tps() {
    let mut tc = TimeControl::new();
    tc.play();
    tc.set_speed(Speed::T30);
    assert_eq!(tc.advance(33), 1);
    assert_eq!(tc.tick_count(), 1);
    assert_eq!(tc.accumulator_ms, 0);
    // 100 ms a 30 tps = 3 ticks (99 ms) + 1 ms de residuo.
    assert_eq!(tc.advance(100), 3);
    assert_eq!(tc.tick_count(), 4);
    assert_eq!(tc.accumulator_ms, 1);
}

/// t-time-06: delta muy grande no desborda (saturación).
#[test]
fn t_time_06_advance_big_delta_no_overflow() {
    let mut tc = TimeControl::new();
    tc.play();
    tc.set_speed(Speed::T2);
    let n = tc.advance(u64::MAX);
    assert!(n > 0);
    assert_eq!(tc.tick_count(), n);
}

/// t-time-07: `advance` sin Running no acumula ni consume.
#[test]
fn t_time_07_advance_not_running_no_consume() {
    let mut tc = TimeControl::new();
    // Idle: no acumula.
    assert_eq!(tc.advance(500), 0);
    assert_eq!(tc.accumulator_ms, 0);
    assert_eq!(tc.tick_count(), 0);

    // Paused: no acumula.
    tc.play();
    tc.pause();
    assert_eq!(tc.advance(500), 0);
    assert_eq!(tc.accumulator_ms, 0);
    assert_eq!(tc.tick_count(), 0);
}

/// t-time-08: transiciones válidas: Idle -> Running(play) -> Paused(pause) -> Running(play).
#[test]
fn t_time_08_valid_transitions() {
    let mut tc = TimeControl::new();
    assert_eq!(tc.state, RunState::Idle);

    tc.play();
    assert!(matches!(tc.state, RunState::Running(_)));

    tc.pause();
    assert!(matches!(tc.state, RunState::Paused(_)));

    tc.play();
    assert!(matches!(tc.state, RunState::Running(_)));
}

/// t-time-09: transiciones inválidas son no-op.
#[test]
fn t_time_09_invalid_transitions_noop() {
    let mut tc = TimeControl::new();
    // pause desde Idle: no-op.
    tc.pause();
    assert_eq!(tc.state, RunState::Idle);
    // play desde Running: no-op (sigue Running).
    tc.play();
    assert!(matches!(tc.state, RunState::Running(_)));
    tc.play();
    assert!(matches!(tc.state, RunState::Running(_)));
}

/// t-time-10: `step_once` avanza 1 tick desde cualquier estado, sin tocar accumulator.
#[test]
fn t_time_10_step_once_any_state() {
    let mut tc = TimeControl::new();
    tc.play();
    tc.set_speed(Speed::T10);
    tc.advance(100); // tick = 1, accumulator = 0
    assert_eq!(tc.tick_count(), 1);
    assert_eq!(tc.accumulator_ms, 0);

    // step_once desde Running: +1 tick, accumulator intacto.
    assert_eq!(tc.step_once(), 1);
    assert_eq!(tc.tick_count(), 2);
    assert_eq!(tc.accumulator_ms, 0);

    tc.pause();
    assert_eq!(tc.step_once(), 1);
    assert_eq!(tc.tick_count(), 3);
    assert_eq!(tc.accumulator_ms, 0);
}

/// t-time-11: `set_speed` cambia la velocidad; en Running se aplica al estado.
#[test]
fn t_time_11_set_speed() {
    let mut tc = TimeControl::new();
    assert_eq!(tc.speed, Speed::T2);
    tc.set_speed(Speed::T30);
    assert_eq!(tc.speed, Speed::T30);
    assert_eq!(tc.speed.ms_per_tick(), 33);

    tc.play();
    tc.set_speed(Speed::T10);
    assert!(matches!(tc.state, RunState::Running(Speed::T10)));
}

/// t-time-12: `PrepPlace` rechazada cuando Running (sin mutar mundo).
#[test]
fn t_time_12_prep_place_rejected_running() {
    let mut app = AppState {
        session: Some(LoadedSession::new(&examples_root())),
        ..Default::default()
    };
    app.update(Msg::PrepEnter(0));
    app.update(Msg::TickPlay); // ahora Running (con sim iniciada)
    let _species_name = first_species_name(&LoadedSession::new(&examples_root()));
    let species_idx = 0;

    // Colocación debe ser ignorada porque el estado es Running.
    app.update(Msg::PrepPlace(species_idx, 1, 0));
    // prep_state sigue existiendo y sin colocaciones (rechazado).
    assert!(app.prep_state.is_some());
    assert_eq!(app.prep_state.as_ref().unwrap().placed_count(), 0);
}

/// t-time-13: `PrepRemove` rechazada cuando Running.
#[test]
fn t_time_13_prep_remove_rejected_running() {
    let mut app = AppState {
        session: Some(LoadedSession::new(&examples_root())),
        ..Default::default()
    };
    app.update(Msg::PrepEnter(0));
    // Colocar una criatura primero (estado Idle).
    let _species_name = first_species_name(&LoadedSession::new(&examples_root()));
    app.update(Msg::PrepPlace(0, 1, 0));
    assert_eq!(app.prep_state.as_ref().unwrap().placed_count(), 1);

    app.update(Msg::TickPlay); // Running
    let count_before = app.prep_state.as_ref().unwrap().placed_count();
    let creatures = app.prep_state.as_ref().unwrap().creatures_in_prep();
    if let Some((pid, _, _, _)) = creatures.first() {
        app.update(Msg::PrepRemove(PlacementId(pid.0)));
    }
    assert_eq!(
        app.prep_state.as_ref().unwrap().placed_count(),
        count_before
    );
}

/// t-time-14: `PrepPlace` rechazada cuando Paused.
#[test]
fn t_time_14_prep_place_rejected_paused() {
    let mut app = AppState {
        session: Some(LoadedSession::new(&examples_root())),
        ..Default::default()
    };
    app.update(Msg::PrepEnter(0));
    app.update(Msg::TickPlay);
    app.update(Msg::TickPause); // Paused
    let _species_name = first_species_name(&LoadedSession::new(&examples_root()));
    app.update(Msg::PrepPlace(0, 1, 0));
    assert_eq!(app.prep_state.as_ref().unwrap().placed_count(), 0);
}

/// t-time-15: `PrepRemove` rechazada cuando Paused.
#[test]
fn t_time_15_prep_remove_rejected_paused() {
    let mut app = AppState {
        session: Some(LoadedSession::new(&examples_root())),
        ..Default::default()
    };
    app.update(Msg::PrepEnter(0));
    app.update(Msg::PrepPlace(0, 1, 0));
    app.update(Msg::TickPlay);
    app.update(Msg::TickPause); // Paused
    let count_before = app.prep_state.as_ref().unwrap().placed_count();
    let creatures = app.prep_state.as_ref().unwrap().creatures_in_prep();
    if let Some((pid, _, _, _)) = creatures.first() {
        app.update(Msg::PrepRemove(PlacementId(pid.0)));
    }
    assert_eq!(
        app.prep_state.as_ref().unwrap().placed_count(),
        count_before
    );
}

/// t-time-16: `step_once` no toca el accumulator.
#[test]
fn t_time_16_step_once_no_touch_accumulator() {
    let mut tc = TimeControl::new();
    tc.play();
    tc.set_speed(Speed::T2);
    tc.advance(250); // accumulator = 250, tick = 0
    assert_eq!(tc.accumulator_ms, 250);
    tc.step_once(); // tick = 1, accumulator intacto
    assert_eq!(tc.tick_count(), 1);
    assert_eq!(tc.accumulator_ms, 250);
}

/// t-time-17: residuo se conserva en accumulator entre llamadas a advance.
#[test]
fn t_time_17_residuo_conservado() {
    let mut tc = TimeControl::new();
    tc.play();
    tc.set_speed(Speed::T2); // 500 ms per tick
    assert_eq!(tc.advance(250), 0);
    assert_eq!(tc.accumulator_ms, 250);
    assert_eq!(tc.advance(250), 1);
    assert_eq!(tc.accumulator_ms, 0);
    assert_eq!(tc.tick_count(), 1);
}

/// t-time-18: humo: 10 ticks vía advance producen los mismos eventos que 10 step_tick directos.
#[test]
fn t_time_18_humo_advance_equals_step_tick() {
    // Construir mundo directamente desde receta de examples (como lo haría start_simulation).
    let session = LoadedSession::new(&examples_root());
    let map = session
        .maps()
        .iter()
        .find(|m| m.name == "Plain with rocks")
        .expect("map 'Plain with rocks'");
    let species = session
        .species()
        .iter()
        .find(|s| s.name == "Talker")
        .expect("species 'Talker'");

    // --- Camino A: 10 step_tick directos sobre el núcleo ---
    let mut world_a = World::from_map(
        map,
        &session.resources().terrains,
        &session.resources().objects,
    );
    let species_rc = Rc::new(species.clone());
    world_a.place_creature(&species_rc, 1, 0).expect("place ok");
    let mut sim_a = Simulation::new(world_a);
    sim_a.set_rng_seed(0);
    let buffer_a = Rc::new(RefCell::new(
        instinct_gui::app::log::LogBuffer::with_capacity(1000),
    ));
    sim_a.world.set_output(crate_port(Rc::clone(&buffer_a)));
    for _ in 0..10 {
        let _ = sim_a.step_tick();
    }
    let events_a: Vec<String> = buffer_a
        .borrow()
        .entries()
        .iter()
        .map(|e| e.line.clone())
        .collect();

    // --- Camino B: 10 ticks vía TimeControl.advance + Simulation en AppState ---
    let mut app = AppState {
        session: Some(session),
        ..Default::default()
    };
    app.update(Msg::PrepEnter(0));
    // Colocar la criatura en prep (estado Idle): usar el índice de "Talker".
    let talker_idx = app
        .session
        .as_ref()
        .unwrap()
        .species_names()
        .iter()
        .position(|n| *n == "Talker")
        .expect("Talker species");
    app.update(Msg::PrepPlace(talker_idx, 1, 0));
    assert_eq!(app.prep_state.as_ref().unwrap().placed_count(), 1);
    // Play arranca la simulación desde la receta.
    app.update(Msg::TickPlay);
    assert!(matches!(app.time.state, RunState::Running(_)));
    assert!(app.sim.is_some());
    // 10 ticks a 2 tps = 5000 ms.
    app.update(Msg::TickAdvance(5000));
    assert_eq!(app.time.tick_count(), 10);
    assert_eq!(app.sim.as_ref().unwrap().world().tick(), 10);
    // Invariante: time.tick == sim.world().tick().
    assert_eq!(
        app.time.tick_count(),
        app.sim.as_ref().unwrap().world().tick()
    );

    let events_b: Vec<String> = app
        .log_buffer
        .as_ref()
        .unwrap()
        .borrow()
        .entries()
        .iter()
        .map(|e| e.line.clone())
        .collect();

    assert_eq!(events_a, events_b, "los eventos deben coincidir");
}

/// Puerto auxiliar para tests (mismo patrón que main.rs).
fn crate_port(buffer: Rc<RefCell<instinct_gui::app::log::LogBuffer>>) -> Box<dyn WorldOutput> {
    struct LocalPort(Rc<RefCell<instinct_gui::app::log::LogBuffer>>);
    impl WorldOutput for LocalPort {
        fn emit(&mut self, event: &instinct_core::event::Event) {
            self.0.borrow_mut().push_event(event);
        }
    }
    Box::new(LocalPort(buffer))
}
