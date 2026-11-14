//! Map drawing tests: test_draw_example_map, test_unknown_character_generates_diagnostic.
//! Bodies copied verbatim from tests/world_tests.rs; no Spanish comments here.

use super::common::*;

#[test]
fn test_draw_example_map() {
    let map = Map {
        name: "Plain with rocks".to_string(),
        rows: vec![
            "....................".chars().collect(),
            "...##...............".chars().collect(),
        ],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![object_rock()];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    let drawing = world.draw().unwrap();
    let lines: Vec<&str> = drawing.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "....................");
    assert_eq!(lines[1], "...##...............");
}

#[test]
fn test_unknown_character_generates_diagnostic() {
    let map = Map {
        name: "Rare".to_string(),
        rows: vec!["....x....".chars().collect()],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![object_rock()];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(
        world
            .diagnostics
            .iter()
            .any(|d| d.line == 2 && d.message.contains('x')),
        "{:?}",
        world.diagnostics
    );
}
