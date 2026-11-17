//! Runtime error bus tests: AC3/I6 (interpreter runtime error produces
//! RuntimeError on the bus).
//! Moved from `tests/interp_tests.rs`; logic identical, comments translated
//! to English.

use super::common::*;

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::Event;
use instinct_core::perceptions::PerceptionCatalog;
use instinct_core::resources::{Map, Terrain};
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world::World;
use std::cell::RefCell;
use std::rc::Rc;

/// Builds a species whose program divides by zero, used to trigger a runtime
/// error during simulation.
fn species_with_div_zero_program() -> Rc<Species> {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text =
        "creature DivZero\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nn = 1 / 0\n";
    let result = compile::compile(text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("header");
    Rc::new(Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    ))
}

#[test]
fn test_interpreter_emits_runtime_error_to_bus() {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 1,
        reserve: 100,
    }];
    let objects = vec![];

    let captured = Rc::new(RefCell::new(Vec::<Event>::new()));
    let captured_clone = Rc::clone(&captured);

    let mut world = World::from_map(&map, &terrains, &objects);
    world.set_output(Box::new(SharedOutput {
        events: captured_clone,
    }));

    let species = species_with_div_zero_program();
    let id = world
        .place_creature(&species, 0, 0)
        .expect("place creature");

    // Verify the creature is alive before creating the simulation.
    assert!(world.creatures.iter().flatten().any(|c| c.id == id));

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(42);

    // The first tick executes `n = 1 / 0`, killing the creature with
    // RuntimeError on the bus.
    let _events = sim.step_tick();

    let captured_events = captured.borrow().clone();
    assert!(
        captured_events
            .iter()
            .any(|e| matches!(e, Event::RuntimeError { .. })),
        "expected a RuntimeError on the bus, got: {captured_events:?}"
    );

    // The RuntimeError must carry tick, species, position, message, and line.
    let runtime_errors: Vec<&Event> = captured_events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert!(!runtime_errors.is_empty());
    match &runtime_errors[0] {
        Event::RuntimeError {
            tick,
            species,
            pos,
            message,
            line,
        } => {
            assert_eq!(*tick, 0, "tick of the RuntimeError");
            assert_eq!(species, "DivZero", "species of the RuntimeError");
            assert_eq!(pos.x, 0, "x position of the RuntimeError");
            assert_eq!(pos.y, 0, "y position of the RuntimeError");
            assert!(!message.is_empty(), "RuntimeError message not empty");
            assert_eq!(*line, 7, "error line: n = 1 / 0 is on line 7");
        }
        _ => unreachable!(),
    };
    // The creature must be dead.
    assert!(sim.world.creatures.iter().flatten().all(|c| c.id != id));
}
