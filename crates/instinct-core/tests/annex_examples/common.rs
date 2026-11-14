//! Shared helpers for the annex example integration tests.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::Event;
use instinct_core::perceptions::PerceptionCatalog;
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world::World;
use std::rc::Rc;

pub(crate) fn examples_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
}

/// Compiles an annex `.ins` and returns its `Species` with its program,
/// wrapped in `Rc` to share it through the world catalog (Flyweight).
pub(crate) fn load_annex_creature(name: &str) -> Rc<Species> {
    let path = format!("{}/creatures/{}.ins", examples_dir(), name);
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{name}.ins must exist"));
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let result = compile::compile(&content, &cat, &pc, &path);
    assert!(
        result.diagnostics.is_empty(),
        "{name}.ins should compile without diagnostics, got: {:?}",
        result.diagnostics
    );
    let header = result
        .header
        .unwrap_or_else(|| panic!("{name}.ins should have a header"));
    Rc::new(Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    ))
}

pub(crate) fn grass_terrain() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

pub(crate) fn empty_map() -> Map {
    Map {
        name: "test".to_string(),
        rows: vec![
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
        ],
    }
}

pub(crate) fn build_world(map: &Map, terrains: &[Terrain], objects: &[Object]) -> World {
    let world = World::from_map(map, terrains, objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

pub(crate) fn species_no_program(name: &str, health: i64, lifespan: i64) -> Rc<Species> {
    Rc::new(Species {
        name: name.to_string(),
        faction: "any".to_string(),
        health,
        vision: 3,
        lifespan,
        program: None,
    })
}

/// Runs `n` ticks and returns all emitted events.
pub(crate) fn run_ticks(sim: &mut Simulation, n: u64) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..n {
        all.extend(sim.step_tick());
    }
    all
}

pub(crate) fn has_runtime_error(events: &[Event]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::RuntimeError { .. }))
}
