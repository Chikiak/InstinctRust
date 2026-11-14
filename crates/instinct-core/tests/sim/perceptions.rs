//! Perception scenario tests for the Simulation.
//! Extracted from `tests/sim_tests.rs`: AC6 (self/world perceptions) and
//! AC8 (see/name with integer arguments run; text argument kills).
//! Logic identical to the original; comments and assert messages translated
//! to English.

use instinct_core::event::Event;
use instinct_core::sim::Simulation;

use super::common::*;

// ---------------------------------------------------------------------------
// AC6: a compiled creature reads its own and world perceptions and says them
// ---------------------------------------------------------------------------

#[test]
fn test_simulation_reads_self_and_world_perceptions() {
    // X (faction A) says its species, its faction, and how many enemies it
    // sees. Two creatures of faction B surround it: total_enemies must be 2.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_x = species_with_program_and_faction(
        "X",
        "A",
        100,
        1000,
        "say(species)\nsay(faction)\nsay(total_enemies)\nwait(1)\ngoto start\n",
    );
    let _id_x = world.place_creature(&sp_x, 0, 0).expect("place X");
    let sp_b = species_no_program("B", 100, 1000);
    let _id_b1 = world.place_creature(&sp_b, 1, 0).expect("place B 1");
    let _id_b2 = world.place_creature(&sp_b, 2, 0).expect("place B 2");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);
    let events = sim.step_tick();

    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "X"),
        "says its species, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "A"),
        "says its faction, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "2"),
        "says total_enemies (2), got: {said:?}"
    );
}

// ---------------------------------------------------------------------------
// AC8: see/name with integer arguments run without error; text argument kills
// ---------------------------------------------------------------------------

#[test]
fn test_see_with_text_argument_kills_creature() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp =
        species_with_program_and_faction("D", "A", 100, 1000, "if see(\"a\", 0) == 0 goto start\n");
    let _id = world.place_creature(&sp, 0, 0).expect("place D");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    let errors: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert_eq!(errors.len(), 1, "one runtime error, got: {errors:?}");
    for err in &errors {
        if let Event::RuntimeError { line, .. } = err {
            assert_eq!(*line, 7, "the error names its line: {err:?}");
        }
    }

    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { cause, .. } if cause == "error"))
        .collect();
    assert_eq!(died.len(), 1, "the creature dies from the runtime error");
}

#[test]
fn test_creature_uses_see_and_name() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_with_program_and_faction(
        "Watcher",
        "A",
        100,
        1000,
        "say(see(1, 0))\nsay(name(1, 0))\nwait(1)\ngoto start\n",
    );
    let _id = world.place_creature(&sp, 0, 0).expect("place watcher");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);
    let events = sim.step_tick();

    assert!(
        !events.iter().any(|e| matches!(e, Event::RuntimeError { .. })),
        "no runtime errors, got: {events:?}"
    );
    assert!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .any(|c| c.species.name == "Watcher"),
        "the creature stays alive"
    );

    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(said.len() >= 2, "says see and name, got: {said:?}");
}
