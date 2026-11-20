//! T5: selection over maps and species via `Msg`.

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
