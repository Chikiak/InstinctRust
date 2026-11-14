//! Shared helpers and re-exports for the `world` test submodules.
//! Bodies copied verbatim from tests/world_tests.rs lines 13-72.

pub use instinct_core::actions::ActionResult;
pub use instinct_core::event::{Event, Pos};
pub use instinct_core::resources::{Map, Object, Terrain};
pub use instinct_core::sim::Simulation;
pub use instinct_core::species::Species;
pub use instinct_core::value::Value;
pub use instinct_core::world::{CellContent, CreatureId, CreatureMemento, CreatureState, World};
pub use std::collections::HashMap;
pub use std::rc::Rc;

pub fn vision_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
        ],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![];
    World::from_map(&map, &terrains, &objects)
}

/// Builds a species with the given vision, ready to be shared via `Rc`.
/// Extrinsic state (health, age, position...) stays on the creature; the
/// vision is intrinsic species data and is not mutated after sharing.
pub fn species_with_vision(name: &str, vision: i64) -> Rc<Species> {
    Rc::new(Species {
        name: name.to_string(),
        faction: String::new(),
        health: 0,
        vision,
        lifespan: 0,
        program: None,
    })
}

pub fn terrain_grass() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

pub fn object_rock() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

pub fn empty_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 3], vec!['.'; 3], vec!['.'; 3]],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![object_rock()];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}
