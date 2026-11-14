//! place_creature tests: test_place_rejects_invalid_cells,
//! test_place_creature_returns_stable_id_and_reuses_slot.
//! Bodies copied verbatim from tests/world_tests.rs; Spanish comments
//! translated to concise English. Assertion strings untouched.

use super::common::*;

#[test]
fn test_place_rejects_invalid_cells() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));

    // Outside the map.
    let err = world.place_creature(&species, 5, 5).unwrap_err();
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
        .place_creature(&species, 1, 1)
        .expect("place first creature");
    assert_eq!(world.creatures.len(), 1);
    let err = world.place_creature(&species, 1, 1).unwrap_err();
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
}

#[test]
fn test_place_creature_returns_stable_id_and_reuses_slot() {
    let mut world = empty_world();
    let id1 = world
        .place_creature(&Rc::new(Species::new("A")), 0, 0)
        .expect("place A");
    let id2 = world
        .place_creature(&Rc::new(Species::new("B")), 2, 2)
        .expect("place B");
    assert_eq!(id1, CreatureId(0));
    assert_eq!(id2, CreatureId(1));
    assert_ne!(id1, id2);
    // Creatures are looked up O(1) by stable id.
    assert_eq!(
        world.creature_position(id1).unwrap(),
        instinct_core::event::Pos { x: 0, y: 0 }
    );
    assert_eq!(
        world.creature_position(id2).unwrap(),
        instinct_core::event::Pos { x: 2, y: 2 }
    );

    // Removing id1 frees its slot without touching id2.
    world.remove_creature(id1).expect("remove A");
    assert!(world.creature_position(id1).is_err());
    assert_eq!(
        world.creature_position(id2).unwrap(),
        instinct_core::event::Pos { x: 2, y: 2 }
    );
    assert_eq!(
        world
            .creatures
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|_| i))
            .collect::<Vec<_>>(),
        vec![1]
    );

    // A subsequent place reuses that same id (0).
    let id3 = world
        .place_creature(&Rc::new(Species::new("C")), 0, 0)
        .expect("place C");
    assert_eq!(id3, id1, "the free slot is reused with the same id");
    assert_ne!(id3, id2);
    assert_eq!(
        world.creature_position(id3).unwrap(),
        instinct_core::event::Pos { x: 0, y: 0 }
    );
    // The other creature is not corrupted.
    assert_eq!(
        world.creature_position(id2).unwrap(),
        instinct_core::event::Pos { x: 2, y: 2 }
    );
    assert_eq!(
        world.creatures.iter().filter(|c| c.is_some()).count(),
        2,
        "two live creatures after reusing the slot"
    );
}
