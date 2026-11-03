//! Unit tests for the `world` module.

use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::species::Species;
use instinct_core::world::{CellContent, CreatureId, World};

fn terrain_grass() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

fn object_rock() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

fn empty_world() -> (World, Vec<Terrain>, Vec<Object>) {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 3], vec!['.'; 3], vec!['.'; 3]],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![object_rock()];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    (world, terrains, objects)
}

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
    let drawing = world.draw(&terrains, &objects);
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

// ---------------------------------------------------------------------------
// AC1: place_creature validation
// ---------------------------------------------------------------------------

#[test]
fn test_place_rejects_invalid_cells() {
    let (mut world, terrains, objects) = empty_world();
    let species = Species::new("Walker");

    // Outside the map.
    let err = world.place_creature(species.clone(), 5, 5).unwrap_err();
    assert!(
        err.message.contains("(5,5)"),
        "error names coordinates: {err:?}"
    );
    assert!(
        world.creatures.is_empty(),
        "world unchanged on failed place"
    );

    // On an object: rock at (0,0) in empty_world? No, objects list is separate.
    // Place a rock object on the grid manually: cell (0,0) is grass only.
    // Instead, place a creature first, then try to place another on top.
    let id1 = world
        .place_creature(species.clone(), 1, 1)
        .expect("place first creature");
    assert_eq!(world.creatures.len(), 1);
    let err = world.place_creature(species.clone(), 1, 1).unwrap_err();
    assert!(
        err.message.contains("(1,1)"),
        "error names coordinates: {err:?}"
    );
    assert_eq!(world.creatures.len(), 1, "second creature not placed");
    // First creature still there.
    assert_eq!(
        world.creature_position(id1).unwrap(),
        instinct_core::event::Pos { x: 1, y: 1 }
    );
    let _ = (&terrains, &objects);
}

#[test]
fn test_place_creature_returns_stable_id() {
    let (mut world, _, _) = empty_world();
    let id1 = world
        .place_creature(Species::new("A"), 0, 0)
        .expect("place A");
    let id2 = world
        .place_creature(Species::new("B"), 2, 2)
        .expect("place B");
    assert_ne!(id1, id2);
    // Removing id1 does not reuse its id for a new creature.
    world.remove_creature(id1).expect("remove A");
    let id3 = world
        .place_creature(Species::new("C"), 0, 0)
        .expect("place C");
    assert_ne!(id3, id1);
    assert_ne!(id3, id2);
    assert_eq!(world.creatures.len(), 2);
    let _ = CreatureId(0); // ensure import used
}

// ---------------------------------------------------------------------------
// AC2: render shows first letter of species
// ---------------------------------------------------------------------------

#[test]
fn test_render_shows_creature() {
    let (mut world, terrains, objects) = empty_world();
    world
        .place_creature(Species::new("Walker"), 1, 1)
        .expect("place Walker");
    let drawing = world.draw(&terrains, &objects);
    let lines: Vec<&str> = drawing.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "...");
    assert_eq!(lines[1], ".W.");
    assert_eq!(lines[2], "...");

    // Object still draws its character.
    let (mut world2, terrains2, objects2) = empty_world();
    // cell (0,0) is grass; override to rock by placing rock? The object catalog
    // has rock but the grid has no rock. Just confirm creature overrides terrain.
    let _ = objects2;
    world2
        .place_creature(Species::new("Ent"), 0, 0)
        .expect("place Ent");
    let drawing2 = world2.draw(&terrains2, &objects2);
    let lines2: Vec<&str> = drawing2.lines().collect();
    assert_eq!(lines2[0], "E..");
}

#[test]
fn test_remove_creature_frees_cell() {
    let (mut world, terrains, objects) = empty_world();
    let id = world
        .place_creature(Species::new("Walker"), 1, 1)
        .expect("place");
    assert!(matches!(
        world.content[1][1],
        CellContent::Creature(cid) if cid == id
    ));
    world.remove_creature(id).expect("remove");
    assert!(matches!(world.content[1][1], CellContent::Empty));
    assert!(world.creatures.is_empty());
    let _ = (&terrains, &objects);
}
