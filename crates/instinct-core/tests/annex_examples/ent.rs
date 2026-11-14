//! Ent example tests.

use super::common::*;
use instinct_core::event::{Event, Pos};
use instinct_core::sim::Simulation;

#[test]
fn test_ent_feeds_and_stays() {
    // AC3: Ent feeds on its terrain and stays put.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let ent = load_annex_creature("ent");
    let ent_id = world.place_creature(&ent, 2, 2).expect("place ent");
    // health < 150 so the Ent goes to `roots` and eats.
    world
        .set_creature_health(ent_id, 140)
        .expect("set ent health");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Damaged event of 3 on its own cell.
    assert!(
        events.iter().any(|e| {
            matches!(
                e,
                Event::Damaged {
                    amount, target, ..
                } if *amount == 3 && *target == Pos { x: 2, y: 2 }
            )
        }),
        "ent should damage its own terrain by 3, got: {events:?}"
    );

    // No runtime error.
    assert!(
        !has_runtime_error(&events),
        "no runtime error, got: {events:?}"
    );

    // The Ent did not leave its cell.
    let ent = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == ent_id)
        .expect("ent alive");
    assert_eq!((ent.x, ent.y), (2, 2), "ent should stay on its cell");
}
