//! Shared helpers for `time` tests.

pub use instinct_gui::app::session::LoadedSession;

/// Root of `examples`, resolved from this crate's manifest dir so tests work
/// regardless of the current directory (tests run with cwd = crate root).
pub fn examples_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
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
