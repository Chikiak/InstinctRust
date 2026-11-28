//! T5: selection over maps and species via `Msg`.

use instinct_gui::app::log::LogEntryKind;
use instinct_gui::app::session::LoadedSession;
use instinct_gui::app::{AppState, Msg};

use crate::common::examples_root;

/// T5: initial selection = first map & first species; `Msg::MapSelected(i)` /
/// `Msg::SpeciesSelected(i)` update selection.
#[test]
fn t5_selection_updates_via_messages() {
    let session = LoadedSession::new(&examples_root());
    let first_map = session
        .selected_map()
        .expect("initial selection must be the first map")
        .name
        .clone();
    let first_species = session
        .selected_species()
        .expect("initial selection must be the first species")
        .name
        .clone();

    let mut state = AppState {
        selected: None,
        session: Some(session),
        prep_state: None,
        prep_selected_species: 0,
        time: Default::default(),
        sim: None,
        log_buffer: None,
        last_events: Vec::new(),
    };

    // Initial selection: first map and first species.
    assert_eq!(
        state
            .session
            .as_ref()
            .unwrap()
            .selected_map()
            .map(|m| m.name.as_str()),
        Some(first_map.as_str())
    );
    assert_eq!(
        state
            .session
            .as_ref()
            .unwrap()
            .selected_species()
            .map(|s| s.name.as_str()),
        Some(first_species.as_str())
    );

    // Select a different map / species by index.
    let (map_names, species_names) = {
        let session = state.session.as_ref().unwrap();
        (session.map_names(), session.species_names())
    };
    let map_names: Vec<String> = map_names.iter().map(|s| s.to_string()).collect();
    let species_names: Vec<String> = species_names.iter().map(|s| s.to_string()).collect();

    let map_target = if map_names.len() > 1 { 1 } else { 0 };
    let species_target = if species_names.len() > 1 { 1 } else { 0 };

    state.update(Msg::MapSelected(map_target));
    state.update(Msg::SpeciesSelected(species_target));

    let session = state.session.as_ref().unwrap();
    assert_eq!(
        session.selected_map().map(|m| m.name.as_str()),
        Some(map_names[map_target].as_str())
    );
    assert_eq!(
        session.selected_species().map(|s| s.name.as_str()),
        Some(species_names[species_target].as_str())
    );
}

/// Msg with `session == None` is a no-op (no panic).
#[test]
fn t5_messages_are_noop_without_session() {
    let mut state = AppState::default();
    state.update(Msg::MapSelected(0));
    state.update(Msg::SpeciesSelected(0));
    assert!(state.session.is_none());
    assert_eq!(state.selected, None);
}

/// T6: el brazo `Err(diagnostic)` de `Msg::PrepPlace` no incrementa
/// `placed_count()` y guarda el diagnóstico en `log_buffer` durante la
/// preparación. `PrepPlace` sobre una celda ocupada (roca en `(3, 1)` de
/// "Plain with rocks") y fuera de rango (`(9999, 9999)`) deben fallar sin
/// contar la colocación.
#[test]
fn t6_prep_place_err_no_incrementa_y_guarda_diagnostico() {
    let session = LoadedSession::new(&examples_root());
    let prep_map_idx = session
        .maps()
        .iter()
        .position(|m| m.name == "Plain with rocks")
        .expect("mapa 'Plain with rocks' debe estar cargado en la sesión");
    let species_idx = session
        .species_names()
        .iter()
        .position(|n| *n == "Dwarf")
        .expect("especie 'Dwarf' debe estar en la sesión");

    let mut state = AppState {
        selected: None,
        session: Some(session),
        prep_state: None,
        prep_selected_species: 0,
        time: Default::default(),
        sim: None,
        log_buffer: None,
        last_events: Vec::new(),
    };

    // Entrar en preparación sobre "Plain with rocks".
    state.update(Msg::PrepEnter(prep_map_idx));
    assert!(state.prep_state.is_some());
    assert_eq!(state.prep_state.as_ref().unwrap().placed_count(), 0);

    // (3, 1) está ocupada por una roca del mapa: colocación inválida.
    state.update(Msg::PrepPlace(species_idx, 3, 1));

    // El primer rechazo deja exactamente un diagnóstico cargado en log_buffer.
    {
        let buffer = state.log_buffer.as_ref().expect(
            "log_buffer debe guardar el diagnóstico durante la preparación (celda ocupada)",
        );
        let borrow = buffer.borrow();
        let entries = borrow.entries();
        assert_eq!(
            entries.len(),
            1,
            "el primer PrepPlace inválido debe dejar exactamente un diagnóstico"
        );
        assert_eq!(
            entries[0].kind,
            LogEntryKind::Load,
            "el diagnóstico de colocación inválida debe ser de tipo Load"
        );
        assert!(
            !entries[0].line.trim().is_empty(),
            "el diagnóstico no debe quedar con texto vacío"
        );
        assert!(
            entries[0].line.contains("(3,1)") && entries[0].line.contains("occupied"),
            "el primer diagnóstico debe nombrar la celda (3,1) y el motivo (ocupada): {}",
            entries[0].line
        );
    }

    // (9999, 9999) está fuera del mapa: colocación inválida.
    state.update(Msg::PrepPlace(species_idx, 9999, 9999));

    // Ninguna colocación inválida incrementa el contador de colocaciones.
    assert_eq!(
        state.prep_state.as_ref().unwrap().placed_count(),
        0,
        "colocación inválida no debe incrementar placed_count()"
    );

    // Durante la preparación ambos diagnósticos deben quedar en log_buffer.
    let buffer = state.log_buffer.as_ref().expect(
        "log_buffer debe estar inicializado durante la preparación y guardar el diagnóstico",
    );
    let borrow = buffer.borrow();
    let entries = borrow.entries();
    assert_eq!(
        entries.len(),
        2,
        "se esperaba un diagnóstico por cada colocación inválida (ocupada y fuera de rango)"
    );
    for entry in entries {
        assert_eq!(
            entry.kind,
            LogEntryKind::Load,
            "todo diagnóstico de preparación debe ser de tipo Load"
        );
        assert!(
            !entry.line.trim().is_empty(),
            "el diagnóstico no debe quedar con texto vacío"
        );
        assert_eq!(entry.tick, 0, "un diagnóstico de carga no lleva tick");
    }
    let occupied = entries
        .iter()
        .position(|e| e.line.contains("(3,1)") && e.line.contains("occupied"))
        .expect("falta el diagnóstico de celda ocupada en (3,1)");
    let outside = entries
        .iter()
        .position(|e| e.line.contains("(9999,9999)") && e.line.contains("outside the map"))
        .expect("falta el diagnóstico de fuera de rango en (9999,9999)");
    assert_ne!(
        occupied, outside,
        "los dos diagnósticos deben tener contexto distintivo (ocupada vs fuera de rango)"
    );
}
