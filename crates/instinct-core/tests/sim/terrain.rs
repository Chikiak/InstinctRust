//! Terrain/object end-of-tick tests: AC6 (terrain regeneration, objects with
//! reserve 0 vanish), AC2 (damage then regeneration consistent), AC3
//! (exhausted object removed).
//! Moved from `tests/sim_tests.rs`; logic identical, comments translated to English.

use super::common::*;

use instinct_core::sim::Simulation;
use instinct_core::world::CellContent;

#[test]
fn test_terrain_regenerates_and_empty_objects_vanish() {
    // Map with grass everywhere and a rock at (2,2).
    let mut map = empty_map();
    map.rows[2][2] = '#';
    let rock = rock_object();
    let world = build_world(&map, &[grass_terrain()], std::slice::from_ref(&rock));
    assert!(matches!(world.content[2][2], CellContent::Object(0)));

    let mut sim = Simulation::new(world);
    // Exhaust the object and zero the terrain reserve on the world.
    sim.world.objects[0].reserve = 0;
    sim.world.terrains[0].reserve = 0;

    let _ = sim.step_tick();

    // Terrain at (0,0) with reserve 0, regen 2 -> now 2 (capped at 100).
    assert_eq!(sim.world.terrains[0].reserve, 2);
    // Object at (2,2) with reserve 0 -> gone at end of tick.
    assert!(matches!(sim.world.content[2][2], CellContent::Empty));
}

/// AC2: end-of-tick damage and regeneration read and write the same reserve;
/// regenerating over an intact copy would restore the original value.
#[test]
fn test_terrain_damage_then_regen_consistent() {
    let map = empty_map();
    let world = build_world(&map, &[grass_terrain()], &[]);
    let mut sim = Simulation::new(world);

    // Damage the terrain at (2,2): 100 -> 70.
    sim.world
        .damage_cell_content(2, 2, 30)
        .expect("damage terrain at (2,2)");
    assert_eq!(sim.world.terrains[0].reserve, 70);

    // End of tick: regenerate +2 over the damaged reserve: 72 (not 100).
    let _ = sim.step_tick();
    assert_eq!(
        sim.world.terrains[0].reserve, 72,
        "regeneration starts from the damaged reserve"
    );

    // Second round: damage the same reserve again and regenerate over it.
    sim.world
        .damage_cell_content(2, 2, 30)
        .expect("damage terrain at (2,2) again");
    assert_eq!(sim.world.terrains[0].reserve, 42);
    let _ = sim.step_tick();
    assert_eq!(sim.world.terrains[0].reserve, 44);
}

/// AC3: an object damaged to 0 or less disappears at end of tick and frees its cell.
#[test]
fn test_object_exhausted_still_removed() {
    let mut map = empty_map();
    map.rows[2][2] = '#';
    let rock = rock_object();
    let world = build_world(&map, &[grass_terrain()], std::slice::from_ref(&rock));
    assert!(matches!(world.content[2][2], CellContent::Object(0)));
    let mut sim = Simulation::new(world);

    // Exhaust the object at (2,2) against the world reserve: 100 -> 0.
    sim.world
        .damage_cell_content(2, 2, 100)
        .expect("exhaust object at (2,2)");
    assert_eq!(sim.world.objects[0].reserve, 0);

    let _ = sim.step_tick();

    assert!(
        matches!(sim.world.content[2][2], CellContent::Empty),
        "the exhausted object disappears and frees its cell"
    );
}
