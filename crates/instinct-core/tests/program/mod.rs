//! Unit tests for compiled types core: `Program`, `Line` and the
//! label table (`labels`).
//!
//! Programs are really built with `compile`; the core is not mocked.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::perceptions::PerceptionCatalog;

mod line_clone;
mod resolve_label;

fn default_catalogs() -> (Catalog, PerceptionCatalog) {
    (Catalog::default(), PerceptionCatalog::default())
}

fn compile_text(text: &str) -> instinct_core::compile::CompileResult {
    let (cat, pc) = default_catalogs();
    compile::compile(text, &cat, &pc, "test.ins")
}
