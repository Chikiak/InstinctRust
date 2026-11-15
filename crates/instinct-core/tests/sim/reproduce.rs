//! Reproduce scenario tests, moved from `tests/sim_tests.rs`.
//! Comments translated to English; test logic is identical to the original.

use super::common::*;

use instinct_core::event::Event;
use instinct_core::sim::Simulation;
use instinct_core::world::CreatureState;

#[test]
fn test_reproduce_offspring_starts_at_next_tick() {
    // Mother: start: reproduce(0, 1, 5) goto start
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_with_program("Walker", 100, 1000, "reproduce(0, 1, 5)\ngoto start\n");
    let mother = world.place_creature(&sp, 1, 1).expect("place mother");
    let mut sim = Simulation::new(world);

    // Tick 0: the mother gives birth; the child is born at (1,2) but the
    // step_tick snapshot excludes it from this tick: pc stays at 0.
    let events = sim.step_tick();
    assert!(
        events.iter().all(|e| !matches!(e, Event::Died { .. })),
        "no deaths expected on the birth tick, got: {events:?}"
    );
    let child = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("the child must exist after reproducing");
    assert_eq!((child.x, child.y), (1, 2));
    assert_eq!(child.health, 4, "life = health (5) minus 1 at end of tick");
    assert_eq!(child.age, 1, "age 0 at birth, +1 at end of tick");
    assert_eq!(child.state, CreatureState::Awake);
    assert_eq!(
        child.pc, 0,
        "not executed yet: starts at start on the next tick"
    );
    assert!(
        child.variables.is_empty(),
        "the child is born without variables"
    );

    // Tick 1: the child executes its first turn from start: (label + reproduce).
    let events = sim.step_tick();
    assert!(
        events.iter().all(|e| !matches!(e, Event::Died { .. })),
        "no deaths expected on the second tick, got: {events:?}"
    );
    let child = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("the child is still alive");
    assert_eq!(
        child.pc, 2,
        "the child executed start: and reproduce on its first turn"
    );
    assert_eq!(child.age, 2);
    assert_eq!(child.health, 3);
    // Its own reproduce aborted (4 is not > 5): no second child.
    assert_eq!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .filter(|c| c.id != mother)
            .count(),
        1,
        "only one child (the second reproduce aborted due to insufficient life or occupied cell)"
    );
}
