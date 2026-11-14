//! Consume action tests.

use super::common::*;
use instinct_core::actions::Consume;
use instinct_core::event::Event;
use instinct_core::value::Value;
use instinct_core::world::CellContent;

#[test]
fn test_catalog_resolves_consume() {
    let cat = Catalog::default();
    let cons = cat.resolve("consume", 4).expect("consume arity 4");
    assert_eq!(cons.name(), "consume");
    assert_eq!(cons.arity(), 4);
    assert!(cons.consumes_turn());
}

#[test]
fn test_consume_damages_creature() {
    // AC1: consume against creature reduces its life; <=0 dies on the spot.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 5);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(10), Value::Int(3)],
        )
        .expect("consume applies");
    assert_eq!(r.sleep_turns, 0);
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 10)),
        "expected Damaged, got {:?}",
        r.events
    );
    assert!(
        world.creatures.iter().flatten().all(|c| c.id != victim),
        "victim must be removed"
    );
    assert!(matches!(world.content[0][1], CellContent::Empty));
    // Attacker healed by 3 (no cap).
    assert_eq!(creature_health(&world, attacker), 13, "attacker healed");
}

#[test]
fn test_consume_heals_attacker() {
    // AC2: heals the attacker by heal, no cap.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 10);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(3), Value::Int(7)],
        )
        .expect("consume applies");
    assert_eq!(creature_health(&world, victim), 7);
    assert_eq!(creature_health(&world, attacker), 17, "attacker +7 no cap");
    assert!(r
        .events
        .iter()
        .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 3)),);
}

#[test]
fn test_consume_damages_object() {
    // AC3: object reduces its reserve; at 0 disappears at end of tick.
    let (mut world, _, _) = rock_world();
    let attacker = place_creature_with_health(&mut world, 0, 1, 10);

    let consume = Consume;
    let _ = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(30), Value::Int(5)],
        )
        .expect("consume applies");
    assert_eq!(world.objects[0].reserve, 70);
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    assert!(
        creature_health(&world, attacker) == 15,
        "attacker healed +5"
    );
}

#[test]
fn test_consume_damages_terrain() {
    // AC4: terrain reduces its reserve, including (0,0) under the creature itself.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 1, 1, 10);

    let consume = Consume;
    let _ = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(-1),
                Value::Int(0),
                Value::Int(40),
                Value::Int(10),
            ],
        )
        .expect("consume applies");
    assert_eq!(world.terrains[0].reserve, 60);
    assert_eq!(creature_health(&world, attacker), 20);
}

#[test]
fn test_consume_out_of_map_noop() {
    // AC5: out of map no-op, no damage nor healing, turn consumed.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(-1),
                Value::Int(0),
                Value::Int(100),
                Value::Int(50),
            ],
        )
        .expect("consume applies");
    assert_eq!(r.sleep_turns, 0);
    assert!(r.events.is_empty(), "no events, got {:?}", r.events);
    assert_eq!(
        creature_health(&world, attacker),
        10,
        "no healing out of map"
    );
}

#[test]
fn test_consume_bad_direction_errors() {
    // AC6: dx/dy outside {-1,0,1} error naming the argument.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let consume = Consume;

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(2), Value::Int(0), Value::Int(1), Value::Int(0)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "dx error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Int(-2), Value::Int(1), Value::Int(0)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "dy error: {err:?}");
}

#[test]
fn test_consume_text_argument_errors() {
    // AC7: text argument error naming the argument.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let consume = Consume;

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Text("uno".to_string()),
                Value::Int(0),
                Value::Int(1),
                Value::Int(0),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "text dx error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Text("dos".to_string()),
                Value::Int(1),
                Value::Int(0),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "text dy error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Int(0),
                Value::Text("tres".to_string()),
                Value::Int(0),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("damage"), "text damage error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Int(0),
                Value::Int(1),
                Value::Text("cuatro".to_string()),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("heal"), "text heal error: {err:?}");
}
