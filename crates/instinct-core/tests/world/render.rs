//! Render tests: test_render_shows_creature, test_remove_creature_frees_cell.
//! Bodies copied verbatim from tests/world_tests.rs; Spanish comments
//! translated to concise English. Assertion strings untouched.

use super::common::*;

#[test]
fn test_render_shows_creature() {
    let mut world = empty_world();
    world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place Walker");
    let drawing = world.draw().unwrap();
    let lines: Vec<&str> = drawing.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "...");
    assert_eq!(lines[1], ".W.");
    assert_eq!(lines[2], "...");

    // Object still draws its character.
    let mut world2 = empty_world();
    // The world catalog has rock, but the grid has no objects:
    // this only confirms that the creature draws over the terrain.
    world2
        .place_creature(&Rc::new(Species::new("Ent")), 0, 0)
        .expect("place Ent");
    let drawing2 = world2.draw().unwrap();
    let lines2: Vec<&str> = drawing2.lines().collect();
    assert_eq!(lines2[0], "E..");
}

#[test]
fn test_remove_creature_frees_cell() {
    let mut world = empty_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place");
    assert!(matches!(
        world.content[1][1],
        CellContent::Creature(cid) if cid == id
    ));
    world.remove_creature(id).expect("remove");
    assert!(matches!(world.content[1][1], CellContent::Empty));
    assert!(world.creatures[id.0].is_none(), "the slot was freed");
    assert!(
        world.creatures.iter().all(|c| c.is_none()),
        "the store only has free slots"
    );
}
