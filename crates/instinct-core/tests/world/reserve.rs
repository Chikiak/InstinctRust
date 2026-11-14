//! Reserve consistency tests:
//! test_terrain_reserve_consistent_world_simulation,
//! test_object_reserve_consistent_world_simulation.
//! Bodies copied verbatim from tests/world_tests.rs; Spanish comments
//! translated to concise English. Assertion strings untouched.

use super::common::*;

#[test]
fn test_terrain_reserve_consistent_world_simulation() {
    let mut world = empty_world();
    // (2,2) is grass from the world's single catalog: reserve 100.
    assert_eq!(world.terrains[0].reserve, 100);
    assert_eq!(world.reserve_at(2, 2), Some(100));

    // Damage on the world's catalog (the same one strike_cell mutates).
    world
        .damage_cell_content(2, 2, 30)
        .expect("damage terrain at (2,2)");
    assert_eq!(world.terrains[0].reserve, 70);
    assert_eq!(world.reserve_at(2, 2), Some(70));

    // The simulation reads and regenerates on the same catalog: it has no own copy.
    let mut sim = Simulation::new(world);
    let _ = sim.step_tick(); // end of tick: regenerates +2 over the damaged reserve -> 72

    // World.reserve_at and Simulation's reserve match: a single owner.
    assert_eq!(sim.world().reserve_at(2, 2), Some(72));
    assert_eq!(sim.world.terrains[0].reserve, 72);
    assert_eq!(sim.world.tick(), 1);
}

#[test]
fn test_object_reserve_consistent_world_simulation() {
    let mut world = empty_world();
    // Rock from the catalog at (0,0): reserve 100.
    world.content[0][0] = CellContent::Object(0);
    assert_eq!(world.objects[0].reserve, 100);
    assert_eq!(world.object_name_at(0, 0), Some("Rock"));

    // Deplete the object on the world's catalog.
    world
        .damage_cell_content(0, 0, 100)
        .expect("deplete object at (0,0)");
    assert_eq!(world.objects[0].reserve, 0);

    // End of tick in the simulation on the same catalog: the object is removed.
    let mut sim = Simulation::new(world);
    let _ = sim.step_tick();

    assert!(
        matches!(sim.world.content[0][0], CellContent::Empty),
        "the depleted object frees its cell"
    );
    assert_eq!(sim.world.objects[0].reserve, 0);
    assert_eq!(sim.world.object_name_at(0, 0), None);
}
