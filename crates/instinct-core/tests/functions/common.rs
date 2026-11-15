//! Shared helpers for `see`/`name`/constants/creature integration tests.

pub use instinct_core::actions::Catalog;
pub use instinct_core::event::Event;
pub use instinct_core::expr::parse;
pub use instinct_core::interp::InterpreterVisitor;
pub use instinct_core::perceptions::PerceptionCatalog;
pub use instinct_core::resources::{Map, Object, Terrain};
pub use instinct_core::sim::Simulation;
pub use instinct_core::species::Species;
pub use instinct_core::value::Value;
pub use instinct_core::world::{CellContent, CreatureContext, CreatureId, World};
pub use std::cell::RefCell;
pub use std::collections::HashMap;
pub use std::rc::Rc;

pub(crate) fn grass_terrain() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

pub(crate) fn rock_object() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

pub(crate) fn empty_map() -> Map {
    Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    }
}

pub(crate) fn build_world(map: &Map, terrains: &[Terrain], objects: &[Object]) -> World {
    let world = World::from_map(map, terrains, objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

/// Compiles an `.ins` file and returns the `Species` with its program wrapped in
/// `Rc` for sharing through the world catalog (Flyweight).   [translated from Spanish]
pub(crate) fn species_with_program(
    name: &str,
    faction: &str,
    health: i64,
    lifespan: i64,
    body: &str,
) -> Rc<Species> {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text = format!(
        "creature {name}\nfaction {faction}\nhealth {health}\nvision 3\nlifespan {lifespan}\nstart:\n{body}"
    );
    let result = instinct_core::compile::compile(&text, &cat, &pc, "test.ins");
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
