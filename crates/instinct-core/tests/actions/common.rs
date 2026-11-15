//! Shared helpers for action integration tests.

pub use instinct_core::actions::{
    Action, ActionResult, Attack, Catalog, Consume, Move, Reproduce, Roar, Say, Wait,
};
pub use instinct_core::diagnostic::Diagnostic;
pub use instinct_core::event::{Event, Pos};
pub use instinct_core::resources::{Map, Object, Terrain};
pub use instinct_core::species::Species;
pub use instinct_core::value::Value;
pub use instinct_core::world::{CellContent, CreatureId, CreatureState, World};
pub use std::rc::Rc;

pub(crate) fn world_with_grass() -> (World, Vec<Terrain>, Vec<Object>) {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 1,
        reserve: 100,
    }];
    let objects = vec![];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    (world, terrains, objects)
}

pub(crate) fn place_walker(world: &mut World) -> CreatureId {
    world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place creature")
}

pub(crate) fn rock_world() -> (World, Vec<Terrain>, Vec<Object>) {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.', '#', '.', '.', '.'], vec!['.'; 5]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 1,
        reserve: 100,
    }];
    let objects = vec![Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    (world, terrains, objects)
}

/// Places a creature and sets its health (Species::new has health 0).
pub(crate) fn place_creature_with_health(
    world: &mut World,
    x: usize,
    y: usize,
    health: i64,
) -> CreatureId {
    let species = Rc::new(Species::new("Walker"));
    let id = world
        .place_creature(&species, x, y)
        .expect("place creature");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = health;
        }
    }
    id
}

pub(crate) fn creature_health(world: &World, id: CreatureId) -> i64 {
    world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id)
        .map(|c| c.health)
        .expect("creature exists")
}

/// Applies `Reproduce` to `mother` targeting `(mother.x + dx, mother.y + dy)`
/// with the given `health` for the child. Panics if the action fails to apply.
pub(crate) fn apply_reproduce(
    world: &mut World,
    mother: CreatureId,
    dx: i64,
    dy: i64,
    health: i64,
) -> ActionResult {
    Reproduce
        .apply(
            world,
            mother,
            &[Value::Int(dx), Value::Int(dy), Value::Int(health)],
        )
        .expect("reproduce applies")
}
