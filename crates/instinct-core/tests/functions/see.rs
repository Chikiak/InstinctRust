//! `see` function tests (AC1, AC2).

use super::common::*;

fn see_at(world: &World, id: CreatureId, x: i64, y: i64) -> Value {
    use instinct_core::expr::parse;
    let expr = parse(
        &format!("see({x}, {y})"),
        1,
        "test.ins",
        Catalog::default().functions(),
    )
    .expect("parse see");
    let ctx = CreatureContext::new(
        world,
        id,
        Rc::new(RefCell::new(HashMap::new())),
    );
    expr.accept(&mut InterpreterVisitor { ctx: &ctx })
        .expect("eval see")
}

// ---------------------------------------------------------------------------
// AC1: see returns GROUND/OBJECT/ALLY/ENEMY per content
// ---------------------------------------------------------------------------

#[test]
fn test_see_returns_constant_per_content() {
    let rock = rock_object();
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[rock]);
    let sp = Rc::new(Species {
        name: "Tester".to_string(),
        faction: "A".to_string(),
        health: 0,
        vision: 3,
        lifespan: 0,
        program: None,
    });
    let id = world.place_creature(&sp, 2, 2).expect("place");

    // GROUND: empty cell inside map and radius (4, 2), distance 2 <= vision 3
    assert_eq!(
        see_at(&world, id, 4, 2),
        Value::Int(0),
        "empty cell is GROUND"
    );

    // OBJECT: rock in the catalog and placed at (0, 0)
    world.content[0][0] = CellContent::Object(0);
    assert_eq!(
        see_at(&world, id, 0, 0),
        Value::Int(1),
        "object cell is OBJECT"
    );

    // ALLY: own cell (2, 2) has the creature itself, same faction
    assert_eq!(see_at(&world, id, 2, 2), Value::Int(2), "own cell is ALLY");

    // ALLY: place ally at (1, 2)
    let ally = Rc::new(Species {
        name: "Ally".to_string(),
        faction: "A".to_string(),
        health: 0,
        vision: 0,
        lifespan: 0,
        program: None,
    });
    world.place_creature(&ally, 1, 2).expect("place ally");
    assert_eq!(see_at(&world, id, 1, 2), Value::Int(2), "ally cell is ALLY");

    // ENEMY: place enemy at (3, 2)
    let enemy = Rc::new(Species {
        name: "Enemy".to_string(),
        faction: "B".to_string(),
        health: 0,
        vision: 0,
        lifespan: 0,
        program: None,
    });
    world.place_creature(&enemy, 3, 2).expect("place enemy");
    assert_eq!(
        see_at(&world, id, 3, 2),
        Value::Int(3),
        "enemy cell is ENEMY"
    );
}

// ---------------------------------------------------------------------------
// AC2: see returns NONE out of radius, OBJECT out of map within radius
// ---------------------------------------------------------------------------

#[test]
fn test_see_outside_radius_and_map() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = Rc::new(Species {
        name: "Tester".to_string(),
        faction: "A".to_string(),
        health: 0,
        vision: 1,
        lifespan: 0,
        program: None,
    });
    let id = world.place_creature(&sp, 2, 2).expect("place");

    // Out of radius: (4, 2) distance = 2 > vision 1 -> NONE (-1)
    assert_eq!(
        see_at(&world, id, 4, 2),
        Value::Int(-1),
        "out of radius is NONE"
    );

    // Out of map but within radius: (-1, 0) distance = 1 <= vision 3 -> OBJECT (1)
    let sp2 = Rc::new(Species {
        name: "Tester2".to_string(),
        faction: "A".to_string(),
        health: 0,
        vision: 3,
        lifespan: 0,
        program: None,
    });
    let id2 = world.place_creature(&sp2, 0, 0).expect("place 2");
    assert_eq!(
        see_at(&world, id2, -1, 0),
        Value::Int(1),
        "out of map within radius is OBJECT"
    );
}
