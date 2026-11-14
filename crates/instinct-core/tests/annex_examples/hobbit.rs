//! Hobbit example tests.

use super::common::*;
use instinct_core::event::Event;
use instinct_core::sim::Simulation;

#[test]
fn test_hobbit_flees_and_speaks() {
    // AC2: Hobbit flees and speaks.
    // Hobbit at (2,2), enemy (other faction) at (3,2): enemy_dist == 1 >= 0.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let hobbit = load_annex_creature("hobbit");
    let hobbit_id = world.place_creature(&hobbit, 2, 2).expect("place hobbit");
    let enemy = species_no_program("Enemy", 20, 100);
    let _enemy_id = world.place_creature(&enemy, 3, 2).expect("place enemy");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Said the flee line.
    assert!(
        events.iter().any(
            |e| matches!(e, Event::Said { text, .. } if text == "they are taking us to Isengard")
        ),
        "hobbit should say the flee line, got: {events:?}"
    );

    // No runtime error.
    assert!(
        !has_runtime_error(&events),
        "no runtime error, got: {events:?}"
    );

    // The hobbit moved away from the enemy (was at (2,2), enemy at (3,2),
    // distance 1; now at (0,2), distance 3).
    let hobbit = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == hobbit_id)
        .expect("hobbit alive");
    assert_eq!(
        (hobbit.x, hobbit.y),
        (0, 2),
        "hobbit should have moved away"
    );
}
