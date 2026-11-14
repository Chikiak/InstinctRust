//! Uruk example tests.

use super::common::*;
use instinct_core::event::{Event, Pos};
use instinct_core::sim::Simulation;

#[test]
fn test_uruk_hunts_and_breeds() {
    // AC5: Uruk hunts and breeds.
    // --- Hunt: Uruk consumes an adjacent enemy -----------------------------
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let uruk = load_annex_creature("uruk");
    let _uruk_id = world.place_creature(&uruk, 0, 0).expect("place uruk");
    let enemy = species_no_program("Enemy", 10, 100);
    let _enemy_id = world.place_creature(&enemy, 1, 0).expect("place enemy");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Consumes the adjacent enemy (damage 15) and the enemy dies.
    assert!(
        events.iter().any(|e| {
            matches!(
                e,
                Event::Damaged {
                    amount, target, ..
                } if *amount == 15 && *target == Pos { x: 1, y: 0 }
            )
        }),
        "uruk should damage the adjacent enemy by 15, got: {events:?}"
    );
    assert!(
        events.iter().any(|e| matches!(e, Event::Died { .. })),
        "enemy should die, got: {events:?}"
    );
    assert!(
        !has_runtime_error(&events),
        "no runtime error while hunting, got: {events:?}"
    );
    assert!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .all(|c| c.species.name != "Enemy"),
        "enemy removed from the map"
    );

    // --- Breed: Uruk alone breeds when conditions hold ---------------------
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let uruk = load_annex_creature("uruk");
    let uruk_id = world.place_creature(&uruk, 2, 2).expect("place uruk");
    world
        .set_creature_health(uruk_id, 80)
        .expect("set uruk health");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);

    // Tick 0: breeds below.
    let events = sim.step_tick();
    assert!(
        !has_runtime_error(&events),
        "no runtime error while breeding, got: {events:?}"
    );
    let child = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != uruk_id)
        .expect("a child should exist");
    assert_eq!((child.x, child.y), (2, 3), "child below the mother");
    assert_eq!(child.species.name, "Uruk");
    let mother = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == uruk_id)
        .expect("mother alive");
    assert!(mother.health < 80, "mother spent health to breed");

    // --- 30 ticks without any RuntimeError ---------------------------------
    let events = run_ticks(&mut sim, 30);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::RuntimeError { .. })),
        "no runtime error in 30 ticks, got: {events:?}"
    );
}
