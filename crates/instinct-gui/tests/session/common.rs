//! Shared helpers for `session` tests.

pub use instinct_gui::app::session::LoadedSession;

/// Root of `examples`, resolved from this crate's manifest dir so tests work
/// regardless of the current directory (tests run with cwd = crate root).
pub fn examples_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}
