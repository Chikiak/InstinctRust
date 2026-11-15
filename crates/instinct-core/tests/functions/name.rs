//! name function tests (AC3).

use super::common::*;

fn name_at(world: &World, id: CreatureId, x: i64, y: i64) -> Value {
    use instinct_core::expr::parse;
    let expr = parse(
        &format!("name({x}, {y})"),
        1,
        "test.ins",
        Catalog::default().functions(),
    )
    .expect("parse name");
    let ctx = CreatureContext::new(
        world,
        id,
        Rc::new(RefCell::new(HashMap::new())),
    );
    expr.accept(&mut InterpreterVisitor { ctx: &ctx })
        .expect("eval name")
}

// ---------------------------------------------------------------------------
// AC3: name returns element names
// ---------------------------------------------------------------------------

#[test]
fn test_name_returns_element_name() {
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
