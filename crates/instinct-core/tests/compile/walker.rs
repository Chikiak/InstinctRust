//! Walker example compilation test (AC1).

use super::*;

#[test]
fn test_compile_walker_example() {
    let path = format!(
        "{}/../../examples/creatures/walker.ins",
        env!("CARGO_MANIFEST_DIR")
    );
    let content = std::fs::read_to_string(&path).expect("walker.ins must exist");
    let (cat, pc) = default_catalogs();
    let result = compile::compile(&content, &cat, &pc, &path);
    assert!(
        result.diagnostics.is_empty(),
        "walker.ins should compile without diagnostics, got: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("walker.ins should have a header");
    assert_eq!(header.species_name, "Walker");
    assert_eq!(header.faction, "any");
    assert_eq!(header.health, 10);
    assert_eq!(header.vision, 3);
    assert_eq!(header.lifespan, 100);
    assert!(
        result.program.resolve_label("start").is_some(),
        "program should have a 'start' label"
    );
}
