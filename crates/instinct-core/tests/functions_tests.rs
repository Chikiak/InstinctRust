//! Integration tests for `see`/`name` functions and constants (F3-03).
//! Uses the real world, the real catalog, and the real evaluation context.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::Event;
use instinct_core::expr::parse;
use instinct_core::perception::PerceptionCatalog;
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::{CellContent, World};

fn grass_terrain() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

fn rock_object() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

fn empty_map() -> Map {
    Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    }
}

fn build_world(map: &Map, terrains: &[Terrain], objects: &[Object]) -> World {
    let world = World::from_map(map, terrains, objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

/// Compiles an `.ins` body and returns a Species with the compiled program.
fn species_with_program(
    name: &str,
    faction: &str,
    health: i64,
    lifespan: i64,
    body: &str,
) -> Species {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text = format!(
        "creature {name}\nfaction {faction}\nhealth {health}\nvision 3\nlifespan {lifespan}\nstart:\n{body}"
    );
    let result = compile::compile(&text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("header");
    Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    )
}

use std::rc::Rc;

// ---------------------------------------------------------------------------
// AC1: see returns GROUND/OBJECT/ALLY/ENEMY per content
// ---------------------------------------------------------------------------

#[test]
fn test_see_returns_constant_per_content() {
    let rock = rock_object();
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[rock]);
    let mut sp = Species::new("Tester");
    sp.faction = "A".to_string();
    sp.vision = 3;
    let id = world.place_creature(sp.clone(), 2, 2).expect("place");

    // GROUND: celda vacía dentro del mapa y del radio (4, 2), distancia 2 <= vision 3
    assert_eq!(
        see_at(&world, id, 4, 2),
        Value::Int(0),
        "empty cell is GROUND"
    );

    // OBJECT: roca en el catálogo y colocada en (0, 0)
    world.content[0][0] = CellContent::Object(0);
    assert_eq!(
        see_at(&world, id, 0, 0),
        Value::Int(1),
        "object cell is OBJECT"
    );

    // ALLY: own cell (2, 2) has the creature itself, same faction
    assert_eq!(see_at(&world, id, 2, 2), Value::Int(2), "own cell is ALLY");

    // ALLY: place ally at (1, 2)
    let mut ally = Species::new("Ally");
    ally.faction = "A".to_string();
    world.place_creature(ally, 1, 2).expect("place ally");
    assert_eq!(see_at(&world, id, 1, 2), Value::Int(2), "ally cell is ALLY");

    // ENEMY: place enemy at (3, 2)
    let mut enemy = Species::new("Enemy");
    enemy.faction = "B".to_string();
    world.place_creature(enemy, 3, 2).expect("place enemy");
    assert_eq!(
        see_at(&world, id, 3, 2),
        Value::Int(3),
        "enemy cell is ENEMY"
    );
}

fn see_at(world: &World, id: instinct_core::world::CreatureId, x: i64, y: i64) -> Value {
    use instinct_core::expr::parse;
    let expr = parse(
        &format!("see({x}, {y})"),
        1,
        "test.ins",
        Catalog::default().functions(),
    )
    .expect("parse see");
    let ctx = instinct_core::world::CreatureContext::new(world, id);
    expr.eval(&ctx).expect("eval see")
}

// ---------------------------------------------------------------------------
// AC2: see returns NONE out of radius, OBJECT out of map within radius
// ---------------------------------------------------------------------------

#[test]
fn test_see_outside_radius_and_map() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let mut sp = Species::new("Tester");
    sp.faction = "A".to_string();
    sp.vision = 1;
    let id = world.place_creature(sp.clone(), 2, 2).expect("place");

    // Out of radius: (4, 2) distance = 2 > vision 1 -> NONE (-1)
    assert_eq!(
        see_at(&world, id, 4, 2),
        Value::Int(-1),
        "out of radius is NONE"
    );

    // Out of map but within radius: (-1, 2) distance = 3 > vision 1... let's use vision 3
    let mut sp2 = Species::new("Tester2");
    sp2.faction = "A".to_string();
    sp2.vision = 3;
    let id2 = world.place_creature(sp2.clone(), 0, 0).expect("place 2");
    // (-1, 0): distance = 1 <= vision 3, out of map -> OBJECT (1)
    assert_eq!(
        see_at(&world, id2, -1, 0),
        Value::Int(1),
        "out of map within radius is OBJECT"
    );
}

// ---------------------------------------------------------------------------
// AC3: name returns element names
// ---------------------------------------------------------------------------

#[test]
fn test_name_returns_element_name() {
    let rock = rock_object();
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[rock]);
    let mut sp = Species::new("Tester");
    sp.faction = "A".to_string();
    sp.vision = 3;
    let id = world.place_creature(sp.clone(), 2, 2).expect("place");

    // name of own cell: species name "Tester"
    assert_eq!(
        name_at(&world, id, 2, 2),
        Value::Text("Tester".to_string()),
        "own cell name is species"
    );

    // name of empty cell: terrain name "Grass"
    assert_eq!(
        name_at(&world, id, 4, 2),
        Value::Text("Grass".to_string()),
        "empty cell name is terrain"
    );

    // name of object cell: object name "Rock"
    world.content[0][0] = CellContent::Object(0);
    assert_eq!(
        name_at(&world, id, 0, 0),
        Value::Text("Rock".to_string()),
        "object cell name is object"
    );

    // Out of radius: empty text
    assert_eq!(
        name_at(&world, id, 10, 10),
        Value::Text("".to_string()),
        "out of radius is empty text"
    );
}

fn name_at(world: &World, id: instinct_core::world::CreatureId, x: i64, y: i64) -> Value {
    use instinct_core::expr::parse;
    let expr = parse(
        &format!("name({x}, {y})"),
        1,
        "test.ins",
        Catalog::default().functions(),
    )
    .expect("parse name");
    let ctx = instinct_core::world::CreatureContext::new(world, id);
    expr.eval(&ctx).expect("eval name")
}

// ---------------------------------------------------------------------------
// AC4: constants compare with see
// ---------------------------------------------------------------------------

#[test]
fn test_constants_compare_with_see() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let mut sp = Species::new("Tester");
    sp.faction = "A".to_string();
    sp.vision = 3;
    let id = world.place_creature(sp.clone(), 2, 2).expect("place");

    // Constant values
    assert_eq!(const_value("NONE"), -1);
    assert_eq!(const_value("GROUND"), 0);
    assert_eq!(const_value("OBJECT"), 1);
    assert_eq!(const_value("ALLY"), 2);
    assert_eq!(const_value("ENEMY"), 3);

    // see(4, 2) on empty cell -> GROUND (0), compare == GROUND (0) -> 1
    let expr = parse(
        "see(4, 2) == GROUND",
        1,
        "test.ins",
        Catalog::default().functions(),
    )
    .expect("parse");
    let ctx = instinct_core::world::CreatureContext::new(&world, id);
    assert_eq!(expr.eval(&ctx).unwrap(), Value::Int(1));

    // see(4, 2) == OBJECT -> 0
    let expr = parse(
        "see(4, 2) == OBJECT",
        1,
        "test.ins",
        Catalog::default().functions(),
    )
    .expect("parse");
    assert_eq!(expr.eval(&ctx).unwrap(), Value::Int(0));
}

fn const_value(name: &str) -> i64 {
    let cat = Catalog::default();
    cat.resolve_constant(name).expect("constant resolves")
}

// ---------------------------------------------------------------------------
// AC8: creature using see/name runs without errors
// ---------------------------------------------------------------------------

#[test]
fn test_creature_uses_see_and_name() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_with_program(
        "Watcher",
        "A",
        100,
        1000,
        "say(see(1, 0))\nsay(name(1, 0))\nwait(1)\ngoto start\n",
    );
    let _id = world.place_creature(sp, 0, 0).expect("place watcher");

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
    // Both see and name should produce output without error.
    assert!(
        said.len() >= 2,
        "creature using see and name should say at least twice, got: {said:?}"
    );
    assert!(
        !said.iter().any(|t| t.is_empty()),
        "outputs should be non-empty, got: {said:?}"
    );
}
