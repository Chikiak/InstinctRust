//! Death scenario tests: AC5 (death by health and by age) and AC3
//! (step_tick iterates only over live ids in the store).
//! Moved from `tests/sim_tests.rs`; logic identical, comments translated to English.

use super::common::*;

use instinct_core::event::Event;
use instinct_core::sim::Simulation;
use instinct_core::world::CreatureId;

#[test]
fn test_death_by_health_and_by_age() {
    // Creature A: health 10, lifespan 100 -> dies by health at end of tick 10.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_a = species_no_program("A", 10, 100);
    let id_a = world.place_creature(&sp_a, 0, 0).expect("place A");

    let mut sim = Simulation::new(world);

    // 9 ticks: alive, health goes 10 -> 1.
    for _ in 0..9 {
        let events = sim.step_tick();
        assert!(
            events.iter().all(|e| !matches!(e, Event::Died { .. })),
            "no death expected before tick 10, got: {events:?}"
        );
    }
    let creature = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id_a)
        .expect("creature alive after 9 ticks");
    assert_eq!(creature.health, 1);
    assert_eq!(creature.age, 9);

    // Tick 10: health 1 -> 0, dies by health.
    let events = sim.step_tick();
    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(died.len(), 1);
    match &died[0] {
        Event::Died {
            tick,
            species,
            cause,
            ..
        } => {
            assert_eq!(*tick, 9); // tick counter is 9 before incrementing to 10
            assert_eq!(species, "A");
            assert_eq!(cause, "vida");
        }
        _ => panic!("expected Died event"),
    }
    assert!(
        sim.world.creatures.iter().flatten().all(|c| c.id != id_a),
        "id_a must have disappeared from the store"
    );

    // Creature B: health 100, lifespan 5 -> dies by age at end of tick 5.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_b = species_no_program("B", 100, 5);
    let id_b = world.place_creature(&sp_b, 0, 0).expect("place B");
    let mut sim = Simulation::new(world);

    for _ in 0..4 {
        let events = sim.step_tick();
        assert!(
            events.iter().all(|e| !matches!(e, Event::Died { .. })),
            "no death expected before tick 5"
        );
    }
    let creature = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id_b)
        .expect("creature alive after 4 ticks");
    assert_eq!(creature.age, 4);

    let events = sim.step_tick();
    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(died.len(), 1);
    match &died[0] {
        Event::Died {
            tick,
            species,
            cause,
            ..
        } => {
            assert_eq!(*tick, 4);
            assert_eq!(species, "B");
            assert_eq!(cause, "edad");
        }
        _ => panic!("expected Died event"),
    }
    assert!(
        sim.world.creatures.iter().flatten().all(|c| c.id != id_b),
        "id_b must have disappeared from the store"
    );
}

// ---------------------------------------------------------------------------
// AC3: step_tick iterates only over live ids in the store (holes ignored).
// ---------------------------------------------------------------------------

#[test]
fn test_sim_iterates_live_ids_only() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_no_program("Walker", 100, 1000);
    let id_a = world.place_creature(&sp, 0, 0).expect("place A");
    let id_b = world.place_creature(&sp, 1, 0).expect("place B");
    let id_c = world.place_creature(&sp, 2, 0).expect("place C");

    // Free the middle hole before the tick: id_b ceases to exist.
    world.remove_creature(id_b).expect("remove B");

    // The store has a hole at id_b; live ids are id_a and id_c.
    let live: Vec<CreatureId> = world
        .creatures
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| slot.as_ref().map(|_| CreatureId(i)))
        .collect();
    assert_eq!(live, vec![id_a, id_c]);

    let mut sim = Simulation::new(world);
    let events = sim.step_tick();

    // No deaths or errors; only the living age one tick.
    assert!(
        !events.iter().any(|e| matches!(e, Event::Died { .. })),
        "no deaths expected, got: {events:?}"
    );
    let after: Vec<(CreatureId, i64)> = sim
        .world
        .creatures
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| slot.as_ref().map(|c| (CreatureId(i), c.age)))
        .collect();
    assert_eq!(after, vec![(id_a, 1), (id_c, 1)]);
    assert!(
        sim.world.creatures[id_b.0].is_none(),
        "id_b's hole is still free after the tick"
    );
}
