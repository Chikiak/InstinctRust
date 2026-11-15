//! Creature state perception tests.

use super::common::*;
use instinct_core::value::Value;

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
