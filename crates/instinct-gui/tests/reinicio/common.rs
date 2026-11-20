//! Shared helpers for `reinicio` tests.

/// Root de `examples`, resuelto desde el manifest dir del crate para que los
/// tests funcionen sin depender del directorio de trabajo actual.
pub fn examples_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// Entra en preparación sobre "Plain with rocks", coloca una Talker en (1, 0)
/// y devuelve la app lista (sim aún no iniciada). Semilla de la receta = 0
/// (PrepEnter la hardcodea en 0).
pub fn enter_prep_with_creature() -> instinct_gui::app::AppState {
    let session = instinct_gui::app::session::LoadedSession::new(&examples_root());
    let mut app = instinct_gui::app::AppState {
        session: Some(session),
        ..Default::default()
    };
    app.update(instinct_gui::app::Msg::PrepEnter(0));
    let talker_idx = app
        .session
        .as_ref()
        .unwrap()
        .species_names()
        .iter()
        .position(|n| *n == "Talker")
        .expect("Talker species");
    app.update(instinct_gui::app::Msg::PrepPlace(talker_idx, 1, 0));
    app
}
