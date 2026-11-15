//! Nearby perception tests.

use super::common::*;
use instinct_core::species::Species;
use std::rc::Rc;

#[test]
fn test_near_counts_exclude_own_cell() {
    let mut world = empty_world(); // 5x5 grass
    let id = world
        .place_creature(&species_with_vision("Walker", 2), 4, 2)
        .expect("place walker");

    // Two allies at (3, 2) and (2, 2) — both distance <= 2
    let _id_ally1 = world
        .place_creature(&Rc::new(Species::new("Ally1")), 3, 2)
        .expect("place ally 1");
    let _id_ally2 = world
        .place_creature(&Rc::new(Species::new("Ally2")), 2, 2)
        .expect("place ally 2");

    // One enemy at (1, 2) — distance 3 from (4, 2), out of vision 2
    // Let me use distance 2 instead: (2, 2) is distance 2, but that's where ally2 is.
    // Place enemy at (0, 2): distance = max(4, 0) = 4 > 2, out of range.
    // Place enemy at (3, 0): distance = max(1, 2) = 2 <= 2, in range.
    let sp_e = Rc::new(Species {
        name: "Enemy".to_string(),
        faction: "B".to_string(),
        health: 0,
        vision: 0,
        lifespan: 0,
        program: None,
    });
    let _id_enemy = world
        .place_creature(&sp_e, 3, 0)
        .expect("place enemy at distance 2");

    // One object: empty_world has no objects on grid. Place one manually.
    // content[row][col]: row=1, col=3 => position (3, 1). Distance from (4,2): max(1, 1) = 1 <= 2.
    world.content[1][3] = instinct_core::world::CellContent::Object(0);

    assert_eq!(
        unwrap_int(eval(&world, id, "allies_near").unwrap()),
        2,
        "two allies within vision, self excluded"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemies_near").unwrap()),
        1,
        "one enemy within vision"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "objects_near").unwrap()),
        1,
        "one object within vision"
    );
}

fn nearest_fixture() -> (World, CreatureId) {
    let mut world = empty_world(); // 5x5 grass
    let id = world
        .place_creature(&species_with_vision("Walker", 3), 4, 2)
        .expect("place walker");

    // Enemy at (1, 2): distance = max(3, 0) = 3 = vision, IN
    let sp_e = Rc::new(Species {
        name: "Enemy".to_string(),
        faction: "B".to_string(),
        health: 0,
        vision: 0,
        lifespan: 0,
        program: None,
    });
    let _id_enemy = world
        .place_creature(&sp_e, 1, 2)
        .expect("place enemy at distance 3");

    // Ally at (3, 2): distance = max(1, 0) = 1
    let _id_ally = world
        .place_creature(&Rc::new(Species::new("Ally")), 3, 2)
        .expect("place ally at distance 1");

    // Object at (4, 0): distance = max(0, 2) = 2
    world.content[0][4] = instinct_core::world::CellContent::Object(0);

    (world, id)
}

#[test]
fn test_nearest_enemy_reports_dist_dx_dy() {
    // Enemy: dist 3, dx -1, dy 0.
    let (world, id) = nearest_fixture();
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dist").unwrap()),
        3,
        "enemy distance"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dx").unwrap()),
        -1,
        "enemy dx"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dy").unwrap()),
        0,
        "enemy dy"
    );
}

#[test]
fn test_nearest_ally_reports_dist_dx_dy() {
    // Ally: dist 1, dx -1, dy 0.
    let (world, id) = nearest_fixture();
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dist").unwrap()),
        1,
        "ally distance"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dx").unwrap()),
        -1,
        "ally dx"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dy").unwrap()),
        0,
        "ally dy"
    );
}

#[test]
fn test_nearest_object_reports_dist_dx_dy() {
    // Object: dist 2, dx 0, dy -1.
    let (world, id) = nearest_fixture();
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dist").unwrap()),
        2,
        "object distance"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dx").unwrap()),
        0,
        "object dx"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dy").unwrap()),
        -1,
        "object dy"
    );
}

#[test]
fn test_nearest_target_none_visible() {
    let mut world = empty_world(); // 5x5 grass, no other creatures
    let id = world
        .place_creature(&species_with_vision("Walker", 1), 4, 2)
        .expect("place walker");

    // No allies, enemies, or objects within vision 1
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dist").unwrap()),
        -1,
        "no enemy: dist -1"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dx").unwrap()),
        0,
        "no enemy: dx 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dy").unwrap()),
        0,
        "no enemy: dy 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dist").unwrap()),
        -1,
        "no ally: dist -1"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dx").unwrap()),
        0,
        "no ally: dx 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dy").unwrap()),
        0,
        "no ally: dy 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dist").unwrap()),
        -1,
        "no object: dist -1"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dx").unwrap()),
        0,
        "no object: dx 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dy").unwrap()),
        0,
        "no object: dy 0"
    );
}
