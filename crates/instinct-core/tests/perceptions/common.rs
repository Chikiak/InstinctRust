//! Shared helpers for perception integration tests.

pub use instinct_core::actions::Catalog;
pub use instinct_core::expr::parse;
pub use instinct_core::interp::InterpreterVisitor;
pub use instinct_core::perceptions::{CreatureView, Perception, PerceptionCatalog};
pub use instinct_core::resources::{Map, Object, Terrain};
pub use instinct_core::species::Species;
pub use instinct_core::value::Value;
pub use instinct_core::world::{CreatureContext, CreatureId, World};
pub use std::cell::RefCell;
pub use std::collections::HashMap;
pub use std::rc::Rc;

pub(crate) fn empty_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }];
    let objects: Vec<Object> = vec![];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

pub(crate) fn place_walker(world: &mut World) -> CreatureId {
    world
        .place_creature(&Rc::new(Species::new("Walker")), 4, 2)
        .expect("place walker")
}

/// Species with given vision, ready to share via `Rc` (Flyweight). Vision is
/// intrinsic data: set at creation and never mutated after.
pub(crate) fn species_with_vision(name: &str, vision: i64) -> Rc<Species> {
    Rc::new(Species {
        name: name.to_string(),
        faction: String::new(),
        health: 0,
        vision,
        lifespan: 0,
        program: None,
    })
}

pub(crate) fn creature_mut(
    world: &mut World,
    id: CreatureId,
) -> &mut instinct_core::world::Creature {
    world
        .creatures
        .iter_mut()
        .flatten()
        .find(|c| c.id == id)
        .expect("creature exists")
}

pub(crate) fn eval(world: &World, id: CreatureId, text: &str) -> Result<Value, String> {
    let expr = parse(text, 1, "test.ins", Catalog::default().functions()).expect("parse");
    let ctx = CreatureContext::new(world, id, Rc::new(RefCell::new(HashMap::new())));
    expr.accept(&mut InterpreterVisitor { ctx: &ctx })
}

pub(crate) fn unwrap_int(v: Value) -> i64 {
    match v {
        Value::Int(n) => n,
        Value::Text(_) => panic!("expected an integer, got {v:?}"),
    }
}
