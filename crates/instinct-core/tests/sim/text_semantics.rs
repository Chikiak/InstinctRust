//! Text semantics scenario tests for the Simulation.
//! Extracted from `tests/sim_tests.rs`: AC7 (end-to-end text semantics).
//! Logic identical to the original; comments and assert messages translated
//! to English.

use instinct_core::event::Event;
use instinct_core::sim::Simulation;

use super::common::*;

// ---------------------------------------------------------------------------
// AC7: end-to-end text semantics
// ---------------------------------------------------------------------------

#[test]
fn test_text_semantics_kill_on_type_error() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    // T: compares texts -> 1 and 0, survives.
    let sp_t = species_with_program_and_faction(
        "T",
        "A",
        100,
        1000,
        "say(\"hola\" == \"hola\")\nsay(\"hola\" == \"mundo\")\nwait(1)\ngoto start\n",
    );
    let _id_t = world.place_creature(&sp_t, 0, 1).expect("place T");

    // D1: text where an integer is expected (action argument) -> dies on line 2.
    let sp_d1 =
        species_with_program_and_faction("D1", "A", 100, 1000, "wait(\"hola\")\ngoto start\n");
    let _id_d1 = world.place_creature(&sp_d1, 1, 1).expect("place D1");

    // D2: text + integer in arithmetic -> dies on line 2.
    let sp_d2 =
        species_with_program_and_faction("D2", "A", 100, 1000, "say(1 + \"a\")\ngoto start\n");
    let _id_d2 = world.place_creature(&sp_d2, 2, 1).expect("place D2");

    // D3: text in ordering comparison -> dies on line 2.
    let sp_d3 = species_with_program_and_faction(
        "D3",
        "A",
        100,
        1000,
        "if \"a\" < 5 goto start\ngoto start\n",
    );
    let _id_d3 = world.place_creature(&sp_d3, 3, 1).expect("place D3");

    // D4: text as numeric argument of move -> dies on line 2.
    let sp_d4 =
        species_with_program_and_faction("D4", "A", 100, 1000, "move(\"a\", 0, 1)\ngoto start\n");
    let _id_d4 = world.place_creature(&sp_d4, 4, 1).expect("place D4");

    // S: say(42) shows the number as text and survives.
    let sp_s =
        species_with_program_and_faction("S", "A", 100, 1000, "say(42)\nwait(1)\ngoto start\n");
    let _id_s = world.place_creature(&sp_s, 0, 0).expect("place S");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(11);
    let events = sim.step_tick();

    // T says "1" and "0": comparing two texts gives 1 or 0.
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "1"),
        "text == text gives 1, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "0"),
        "text != text gives 0, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "42"),
        "say shows the number as text, got: {said:?}"
    );

    // D1..D4 die with a runtime error on their line.
    let errors: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert_eq!(errors.len(), 4, "four runtime errors, got: {errors:?}");
    for err in &errors {
        if let Event::RuntimeError { line, .. } = err {
            // Body: 6 header lines + `start:` -> first body line is 7.
            assert_eq!(*line, 7, "the error names its line: {err:?}");
        }
    }

    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { cause, .. } if cause == "error"))
        .collect();
    assert_eq!(died.len(), 4, "four deaths from error, got: {died:?}");
    for id in [_id_d1, _id_d2, _id_d3, _id_d4] {
        assert!(
            sim.world.creatures.iter().flatten().all(|c| c.id != id),
            "{id:?} must be dead"
        );
    }
    // T and S are still alive.
    assert!(sim
        .world
        .creatures
        .iter()
        .flatten()
        .any(|c| c.species.name == "T"));
    assert!(sim
        .world
        .creatures
        .iter()
        .flatten()
        .any(|c| c.species.name == "S"));
}
