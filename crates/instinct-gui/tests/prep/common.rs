//! Shared helpers for `prep` tests.

pub use instinct_gui::app::prep::PrepState;
pub use instinct_gui::app::session::LoadedSession;

/// Root of `examples`, resolved from this crate's manifest dir so tests work
/// regardless of the current directory (tests run with cwd = crate root).
pub fn examples_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// Builds a `PrepState` over the "Plain with rocks" map from `examples`.
pub fn enter_prep_examples() -> PrepState {
    let session = LoadedSession::new(&examples_root());
    PrepState::enter_prep(&session, "Plain with rocks", 7).expect("enter_prep must succeed")
}

/// Returns the first species name from the session.
pub fn first_species_name(session: &LoadedSession) -> String {
    session
        .species()
        .first()
        .expect("at least one species")
        .name
        .clone()
}
