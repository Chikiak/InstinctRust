//! Vision tests: test_vision_radius_is_chebyshev, test_nearest_tie_break_is_stable.
//! Bodies copied verbatim from tests/world_tests.rs; comments already in English.

use super::common::*;

#[test]
fn test_vision_radius_is_chebyshev() {
    // AC2: Eight neighbor cells at distance 1; cell at distance exactly vision
    // enters; cell at vision + 1 stays out.
    let mut world = vision_world(); // 5x5, all grass
    let id = world
        .place_creature(&species_with_vision("Walker", 1), 2, 2)
        .expect("place center");

    // All 8 neighbors at Chebyshev distance 1 should be within vision 1.
    for (x, y) in [
        (1, 1),
        (1, 2),
        (1, 3),
        (2, 1),
        (2, 3),
        (3, 1),
        (3, 2),
        (3, 3),
    ] {
        world
            .place_creature(&Rc::new(Species::new("Ally")), x, y)
            .expect("place neighbor");
    }
    let nb = world.nearby(id);
    assert_eq!(nb.allies, 8, "all 8 neighbors are within vision 1");

    // Cells at distance 2 should be out of vision 1: (0, 2) is at distance 2.
    let _id_out = world
        .place_creature(&Rc::new(Species::new("Far")), 0, 2)
        .expect("place at distance 2");
    let nb = world.nearby(id);
    assert_eq!(nb.allies, 8, "distance 2 (vision + 1) is out of vision 1");

    // With vision 2, the cell at distance 2 enters.
    let mut world2 = vision_world();
    let id2 = world2
        .place_creature(&species_with_vision("Walker", 2), 2, 2)
        .expect("place center");
    let _id_at_dist2 = world2
        .place_creature(&Rc::new(Species::new("Ally")), 0, 2)
        .expect("place at distance 2");
    let nb2 = world2.nearby(id2);
    assert_eq!(nb2.allies, 1, "cell at distance exactly vision enters");
    assert_eq!(nb2.ally.unwrap().dist, 2);
}

#[test]
fn test_nearest_tie_break_is_stable() {
    let mut world = vision_world(); // 5x5
    let id = world
        .place_creature(&species_with_vision("Walker", 2), 2, 2)
        .expect("place center");

    // Two allies at the same distance (2): (0, 2) has lower id than (4, 2)
    // since it's placed first. Both at Chebyshev distance 2.
    let _id_a1 = world
        .place_creature(&Rc::new(Species::new("Ally1")), 0, 2)
        .expect("place ally at (0,2)");
    let _id_a2 = world
        .place_creature(&Rc::new(Species::new("Ally2")), 4, 2)
        .expect("place ally at (4,2)");

    let nb = world.nearby(id);
    assert_eq!(nb.allies, 2);
    // First placed (id ascending) wins the tie at same distance.
    assert_eq!(nb.ally.unwrap().dist, 2);
    assert_eq!(nb.ally.unwrap().dx, -1, "dx toward (0,2) from (2,2)");
    assert_eq!(nb.ally.unwrap().dy, 0);

    // Verify stability: reading again gives the same result.
    let nb2 = world.nearby(id);
    assert_eq!(nb2.ally, nb.ally);
    assert_eq!(nb2.allies, nb.allies);
}
