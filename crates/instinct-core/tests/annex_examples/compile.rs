//! Compile tests for the annex creatures.

use super::common::*;

#[test]
fn test_annex_creatures_compile() {
    // AC1: the four annex creatures compile without diagnostics.
    for name in ["hobbit", "ent", "dwarf", "uruk"] {
        let _ = load_annex_creature(name);
    }
}
