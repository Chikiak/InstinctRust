//! Constant comparison tests (AC4).

use super::common::*;

fn const_value(name: &str) -> i64 {
    let cat = Catalog::default();
    cat.resolve_constant(name).expect("constant resolves")
}

// ---------------------------------------------------------------------------
// AC4: constants compare with see
// ---------------------------------------------------------------------------

#[test]
fn test_constants_compare_with_see() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = Rc::new(Species {
        name: "Tester".to_string(),
        faction: "A".to_string(),
        health: 0,
        vision: 3,
        lifespan: 0,
        program: None,
    });
    let id = world.place_creature(&sp, 2, 2).expect("place");

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
    let ctx = CreatureContext::new(&world, id, Rc::new(RefCell::new(HashMap::new())));
    assert_eq!(
        expr.accept(&mut InterpreterVisitor { ctx: &ctx }).unwrap(),
        Value::Int(1)
    );

    // see(4, 2) == OBJECT -> 0
    let expr = parse(
        "see(4, 2) == OBJECT",
        1,
        "test.ins",
        Catalog::default().functions(),
    )
    .expect("parse");
    assert_eq!(
        expr.accept(&mut InterpreterVisitor { ctx: &ctx }).unwrap(),
        Value::Int(0)
    );
}
