//! Shared helpers and re-exports for the `compile` test submodules.
//! Copied verbatim from tests/compile_tests.rs lines 1-22.

pub use instinct_core::actions::Catalog;
pub use instinct_core::compile;
pub use instinct_core::perceptions::PerceptionCatalog;
pub use instinct_core::program::Line;

pub fn default_catalogs() -> (Catalog, PerceptionCatalog) {
    (Catalog::default(), PerceptionCatalog::default())
}

pub fn compile_src(
    text: &str,
) -> (
    Vec<instinct_core::diagnostic::Diagnostic>,
    Option<instinct_core::compile::Header>,
) {
    let (cat, pc) = default_catalogs();
    let result = compile::compile(text, &cat, &pc, "test.ins");
    (result.diagnostics, result.header)
}
