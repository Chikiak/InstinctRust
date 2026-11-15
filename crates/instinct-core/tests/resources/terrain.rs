//! Terrain loading tests for the `resources` module.
use super::common::*;

#[test]
fn test_load_terrain_example() {
    let content = "terrain Grass\nchar .\nresource_max 100\nregen 2\n";
    let (result, d) = load_terrain(content, "grass.te");
    assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
    let t = result.expect("must load the terrain");
    assert_eq!(t.name, "Grass");
    assert_eq!(t.character, '.');
    assert_eq!(t.resource_max, 100);
    assert_eq!(t.regen, 2);
    assert_eq!(t.reserve, 100);
}

#[test]
fn test_alphabetical_order_first_terrain() {
    // Two terrains: "b.te" declares "B", "a.te" declares "A". The first in
    // alphabetical file order is "a.te".
    let (result_a, _) = load_terrain("terrain A\nchar a\nresource_max 5\nregen 1\n", "a.te");
    let (_result_b, _) = load_terrain("terrain B\nchar b\nresource_max 5\nregen 1\n", "b.te");
    let ta = result_a.expect("a.te");
    assert_eq!(ta.name, "A");
    assert_eq!(ta.character, 'a');
}
