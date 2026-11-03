//! Integration tests for perceptions and variables (F0-05).
//! Uses the real world, the real parser (F0-03), and the real evaluation
//! context — no fake contexts.

use instinct_core::actions::Catalog;
use instinct_core::expr::parse;
use instinct_core::perception::{CreatureView, Perception};
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::{CreatureContext, CreatureId, World};

fn empty_world() -> World {
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

fn place_walker(world: &mut World) -> CreatureId {
    world
        .place_creature(Species::new("Walker"), 4, 2)
        .expect("place walker")
}

fn creature_mut(world: &mut World, id: CreatureId) -> &mut instinct_core::world::Creature {
    world
        .creatures
        .iter_mut()
        .find(|c| c.id == id)
        .expect("creature exists")
}

fn eval(world: &World, id: CreatureId, text: &str) -> Result<Value, String> {
    let expr = parse(text, 1, "test.ins").expect("parse");
    let ctx = CreatureContext::new(world, id);
    expr.eval(&ctx)
}

fn unwrap_int(v: Value) -> i64 {
    match v {
        Value::Int(n) => n,
        Value::Text(_) => panic!("expected an integer, got {v:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC1: the seven perceptions report the creature's state
// ---------------------------------------------------------------------------

#[test]
fn test_perceptions_report_creature_state() {
    let mut world = empty_world();
    let id = place_walker(&mut world);
    let c = creature_mut(&mut world, id);
    c.health = 10;
    c.age = 3;
    c.lifespan = 50;
    world.ticks = 7;

    let cases: &[(&str, i64)] = &[
        ("health", 10),
        ("age", 3),
        ("lifespan", 50),
        ("x", 4),
        ("y", 2),
        ("tick", 7),
    ];
    for (name, expected) in cases {
        assert_eq!(
            unwrap_int(eval(&world, id, name).unwrap()),
            *expected,
            "perception {name}"
        );
    }

    // random is an integer between 0 and 99
    let v = eval(&world, id, "random").unwrap();
    assert!(
        matches!(v, Value::Int(n) if (0..=99).contains(&n)),
        "random in 0..=99, got {v:?}"
    );
}

// ---------------------------------------------------------------------------
// AC2: expressions read perceptions
// ---------------------------------------------------------------------------

#[test]
fn test_expression_reads_perceptions() {
    let mut world = empty_world();
    let id = place_walker(&mut world);
    let c = creature_mut(&mut world, id);
    c.health = 10;
    c.x = 4;

    // health * 2 + x = 10 * 2 + 4 = 24
    assert_eq!(eval(&world, id, "health * 2 + x").unwrap(), Value::Int(24));

    // tick in a freshly created world is 0
    let mut fresh = empty_world();
    let id2 = place_walker(&mut fresh);
    assert_eq!(eval(&fresh, id2, "tick").unwrap(), Value::Int(0));
}

// ---------------------------------------------------------------------------
// AC3: random advances the generator once per read; deterministic per seed
// ---------------------------------------------------------------------------

#[test]
fn test_random_consumes_one_draw_per_read() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    world.set_rng_seed(42);
    let d1 = unwrap_int(eval(&world, id, "random").unwrap());
    let d2 = unwrap_int(eval(&world, id, "random").unwrap());
    let d3 = unwrap_int(eval(&world, id, "random").unwrap());

    // Same seed reproduces the same sequence.
    world.set_rng_seed(42);
    let e1 = unwrap_int(eval(&world, id, "random").unwrap());
    let e2 = unwrap_int(eval(&world, id, "random").unwrap());
    assert_eq!(e1, d1);
    assert_eq!(e2, d2);

    // Two reads in one expression consume two draws: random + random == d1 + d2,
    // and the following read is d3 (not d2).
    world.set_rng_seed(42);
    let sum = unwrap_int(eval(&world, id, "random + random").unwrap());
    let next = unwrap_int(eval(&world, id, "random").unwrap());
    assert_eq!(sum, d1 + d2);
    assert_eq!(next, d3);
}

// ---------------------------------------------------------------------------
// AC4: assign, read, retype; unassigned variable errors naming it
// ---------------------------------------------------------------------------

#[test]
fn test_variables_assign_read_and_retype() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    world
        .set_variable(id, "score", Value::Int(41))
        .expect("assign score");
    assert_eq!(eval(&world, id, "score").unwrap(), Value::Int(41));

    // A variable can change from integer to text on reassignment.
    world
        .set_variable(id, "score", Value::Text("listo".to_string()))
        .expect("reassign score");
    assert_eq!(
        eval(&world, id, "score").unwrap(),
        Value::Text("listo".to_string())
    );
}

#[test]
fn test_unassigned_variable_error() {
    let mut world = empty_world();
    let id = place_walker(&mut world);
    let err = eval(&world, id, "missing_var").unwrap_err();
    assert!(
        err.contains("missing_var"),
        "error names the variable: {err}"
    );
}

// ---------------------------------------------------------------------------
// AC5: variables are per creature
// ---------------------------------------------------------------------------

#[test]
fn test_variables_are_per_creature() {
    let mut world = empty_world();
    let id1 = place_walker(&mut world);
    let id2 = world
        .place_creature(Species::new("Walker2"), 0, 0)
        .expect("place second walker");

    world
        .set_variable(id1, "energia", Value::Int(1))
        .expect("assign creature 1");
    world
        .set_variable(id2, "energia", Value::Int(2))
        .expect("assign creature 2");

    assert_eq!(eval(&world, id1, "energia").unwrap(), Value::Int(1));
    assert_eq!(eval(&world, id2, "energia").unwrap(), Value::Int(2));
}

// ---------------------------------------------------------------------------
// AC6: cannot assign to a perception or action name
// ---------------------------------------------------------------------------

#[test]
fn test_cannot_assign_perception() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    let err = world
        .set_variable(id, "health", Value::Int(99))
        .unwrap_err();
    assert!(
        err.message.contains("health"),
        "error names health: {err:?}"
    );

    let err = world.set_variable(id, "wait", Value::Int(1)).unwrap_err();
    assert!(err.message.contains("wait"), "error names wait: {err:?}");
}

// ---------------------------------------------------------------------------
// AC7: the catalog accepts an extra perception defined in the test
// ---------------------------------------------------------------------------

#[test]
fn test_catalog_accepts_extra_perception() {
    #[derive(Debug, Clone, Copy)]
    struct Bonus;

    impl Perception for Bonus {
        fn name(&self) -> &str {
            "bonus"
        }
        fn read(&self, _view: &dyn CreatureView) -> Value {
            Value::Int(7)
        }
        fn clone_box(&self) -> Box<dyn Perception> {
            Box::new(*self)
        }
    }

    #[derive(Debug)]
    struct EmptyView;

    impl CreatureView for EmptyView {
        fn health(&self) -> i64 {
            0
        }
        fn age(&self) -> i64 {
            0
        }
        fn lifespan(&self) -> i64 {
            0
        }
        fn x(&self) -> i64 {
            0
        }
        fn y(&self) -> i64 {
            0
        }
        fn tick(&self) -> u64 {
            0
        }
        fn random(&self) -> i64 {
            0
        }
    }

    let mut cat = Catalog::default();
    assert!(
        cat.resolve_perception("bonus").is_none(),
        "bonus not registered yet"
    );
    // The seven built-in perceptions are still present.
    for name in ["health", "age", "lifespan", "x", "y", "tick", "random"] {
        assert!(
            cat.resolve_perception(name).is_some(),
            "{name} still registered"
        );
    }

    assert!(
        cat.register_perception(Box::new(Bonus)).is_ok(),
        "register bonus"
    );
    let view = EmptyView;
    let p = cat.resolve_perception("bonus").expect("bonus resolves");
    assert_eq!(p.read(&view), Value::Int(7));
}
