use super::{creature_health, grass_world, place_with_health};
use instinct_core::actions::{Action, Consume};
use instinct_core::value::Value;

// Core: consume heals the attacker without cap (direct simulation test).
#[test]
fn test_core_consume_heals_without_cap() {
    // AC2 of F1-02: heal accumulates without cap (10 + 7 = 17).
    let mut world = grass_world();
    let attacker = place_with_health(&mut world, 0, 0, 10);
    let victim = place_with_health(&mut world, 1, 0, 10);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(3), Value::Int(7)],
        )
        .expect("consume applies");
    assert_eq!(r.sleep_turns, 0);
    assert_eq!(creature_health(&world, victim), 7, "victim wounded");
    assert_eq!(
        creature_health(&world, attacker),
        17,
        "attacker healed +7 without cap"
    );
}
