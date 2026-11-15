//! Memento scenario tests, extracted from `tests/sim_tests.rs`.
//! A failed turn must not corrupt the world nor leak variables.
//! Keeps all test logic identical; comments translated to English.

use super::common::*;
use instinct_core::event::Event;
use instinct_core::sim::Simulation;
use instinct_core::world::{CellContent, CreatureId};

/// A creature executes `set("x", 1)` and then an action with a text argument
/// that fails (runtime error). The creature must die cleanly and the world
/// must not be corrupted: the other creatures stay alive.
#[test]
fn test_turn_failure_rolls_back_or_dies_cleanly() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    // Creature that fails: counter = 1 then wait with text -> runtime error.
    let sp_fail = species_with_program_and_faction(
        "Failing",
        "A",
        100,
        1000,
        "contador = 1\nwait(\"no es un numero\")\ngoto start\n",
    );
    let id_fail = world
        .place_creature(&sp_fail, 0, 0)
        .expect("place failing creature");

    // Another healthy creature that must stay alive after the error.
    let sp_ok = species_no_program("Ok", 100, 1000);
    let id_ok = world
        .place_creature(&sp_ok, 1, 0)
        .expect("place ok creature");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // There must be a RuntimeError and a Died with cause "error".
    let errors: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert_eq!(errors.len(), 1, "one runtime error, got: {errors:?}");

    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { cause, .. } if cause == "error"))
        .collect();
    assert_eq!(died.len(), 1, "one death by error, got: {died:?}");

    // The failing creature must have disappeared from the store.
    assert!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .all(|c| c.id != id_fail),
        "the failing creature must be dead"
    );

    // The healthy creature stays alive and unchanged.
    let ok = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id_ok)
        .expect("the healthy creature stays alive");
    assert_eq!(ok.health, 99); // end of tick: 100 - 1 health
    assert_eq!(ok.age, 1); // end of tick: +1 age

    // No orphans on the grid.
    for row in &sim.world.content {
        for cell in row {
            if let CellContent::Creature(id) = cell {
                assert!(
                    sim.world.creatures[id.0].is_some(),
                    "the grid must not reference a nonexistent creature {id:?}"
                );
            }
        }
    }
}

/// A creature assigns a variable and then fails. After the failed turn, the
/// variable must not "leak" partially: the creature is dead, so its variable
/// map no longer exists in the world.
#[test]
fn test_variables_not_leaked_on_failed_turn() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    let sp = species_with_program_and_faction(
        "Leaker",
        "A",
        100,
        1000,
        "tesoro = 42\noro = 7\nattack(\"a\", 0, 1)\ngoto start\n",
    );
    let id = world.place_creature(&sp, 0, 0).expect("place leaker");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let _events = sim.step_tick();

    // The creature must have died by runtime error.
    assert!(
        sim.world.creatures.iter().flatten().all(|c| c.id != id),
        "the creature must be dead after the failed turn"
    );

    // No living creature may have the variable "tesoro" (the creature that
    // assigned it no longer exists).
    let leaked: Vec<CreatureId> = sim
        .world
        .creatures
        .iter()
        .flatten()
        .filter(|c| c.variables.contains_key("tesoro"))
        .map(|c| c.id)
        .collect();
    assert!(
        leaked.is_empty(),
        "no living creature should have variable 'tesoro', got: {leaked:?}"
    );
}
