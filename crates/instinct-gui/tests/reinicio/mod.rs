//! Integration tests for `AppState::restart()` (F5-08).
//! Headless: sin ventana, sin eframe.

mod common;

use crate::common::enter_prep_with_creature;
use instinct_gui::app::time::RunState;
use instinct_gui::app::Msg;

/// t_reinicio_01: restart desde sim a mitad de ticks → mundo tick 0, mismas
/// criaturas, semilla preservada (las próximas corridas reproducen misma historia).
#[test]
fn t_reinicio_01_restart_preserva_estado_inicial() {
    let mut app = enter_prep_with_creature();
    // Forzar semilla 7 para probar preservación (PrepEnter hardcodea 0).
    app.prep_state.as_mut().unwrap().recipe.seed = 7;
    assert_eq!(app.prep_state.as_ref().unwrap().recipe.seed, 7);

    // Arrancar simulación y avanzar a mitad de ticks (3 ticks a 2 tps = 1500 ms).
    app.update(Msg::TickPlay);
    assert!(matches!(app.time.state, RunState::Running(_)));
    assert!(app.sim.is_some());
    app.update(Msg::TickAdvance(1500));
    let tick_antes = app.sim.as_ref().unwrap().world().tick();
    assert!(
        tick_antes > 0,
        "esperaba ticks avanzados, fue {}",
        tick_antes
    );

    // Ejecutar reinicio.
    app.update(Msg::Restart);

    // Queda en preparación: prep_state Some, sim None, tiempo Idle, tick 0.
    assert!(
        app.prep_state.is_some(),
        "prep_state debe persistir tras restart"
    );
    assert!(app.sim.is_none(), "sim debe ser None tras restart");
    assert!(
        matches!(app.time.state, RunState::Idle),
        "tiempo debe estar Idle tras restart"
    );
    assert_eq!(
        app.time.tick_count(),
        0,
        "time.tick debe ser 0 tras restart"
    );

    // Mismas criaturas colocadas y semilla preservada.
    let prep = app.prep_state.as_ref().unwrap();
    assert_eq!(prep.recipe.seed, 7, "semilla debe preservarse tras restart");
    assert_eq!(prep.placed_count(), 1, "misma criatura tras restart");
    assert!(
        prep.world.creature_at(1, 0).is_some(),
        "criatura en (1,0) tras restart"
    );

    // Las próximas corridas reproducen la misma historia: volver a arrancar y
    // avanzar la misma cantidad debe llevar al mismo tick.
    app.update(Msg::TickPlay);
    app.update(Msg::TickAdvance(1500));
    let tick_despues = app.sim.as_ref().unwrap().world().tick();
    assert_eq!(
        tick_despues, tick_antes,
        "el reinicio debe reproducir la misma historia (tick {tick_antes} == {tick_despues})"
    );
}

/// t_reinicio_02: tras restart, app queda en PrepState (prep_state Some, sim None).
#[test]
fn t_reinicio_02_queda_en_prep_state() {
    let mut app = enter_prep_with_creature();
    app.update(Msg::TickPlay);
    app.update(Msg::TickAdvance(500));
    assert!(app.sim.is_some());

    app.update(Msg::Restart);

    assert!(app.prep_state.is_some());
    assert!(app.sim.is_none());
}

/// t_reinicio_03: registro contiene entrada indicando reinicio.
#[test]
fn t_reinicio_03_registro_tiene_entrada_de_reinicio() {
    let mut app = enter_prep_with_creature();
    app.update(Msg::TickPlay);
    app.update(Msg::TickAdvance(500));
    let buffer = app.log_buffer.as_ref().unwrap().clone();
    // Leer el tamaño ANTES del reinicio. `restart` no reemplaza el buffer: solo
    // añade una entrada, así que medir `antes` después daría una longitud igual.
    let antes = buffer.borrow().entries().len();

    app.update(Msg::Restart);

    let borrow = app.log_buffer.as_ref().unwrap().borrow();
    let entries = borrow.entries();
    assert!(
        entries.iter().any(|e| e.line.contains("reinicio")),
        "el registro debe indicar el reinicio, entries: {entries:?}"
    );
    assert!(entries.len() > antes, "se debió agregar una entrada al log");
}
