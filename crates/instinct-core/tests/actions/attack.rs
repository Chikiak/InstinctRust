//! Attack action tests.

use super::common::*;
use instinct_core::actions::{Attack, Catalog};
use instinct_core::event::{Event, Pos};
use instinct_core::value::Value;
use instinct_core::world::CellContent;

#[test]
fn test_catalog_resolves_attack() {
    let cat = Catalog::default();
    let atk = cat.resolve("attack", 3).expect("attack arity 3");
    assert_eq!(atk.name(), "attack");
    assert_eq!(atk.arity(), 3);
    assert!(atk.consumes_turn());
}

#[test]
fn test_attack_kills_creature_immediately() {
    // AC1: creature dies on hit, cell left with only terrain, Died causes life this tick.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 5);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(10)],
        )
        .expect("attack applies");
    assert_eq!(r.sleep_turns, 0);
    assert!(
        r.events.len() >= 2,
        "expected Damaged + Died, got {:?}",
        r.events
    );

    // Damaged evento.
    assert!(r
        .events
        .iter()
        .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 10)));
    // Died cause life.
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Died { cause, .. } if cause == "vida")),
        "expected Died cause life, got {:?}",
        r.events
    );
    // Victim removed.
    assert!(
        world.creatures.iter().flatten().all(|c| c.id != victim),
        "victim must be removed"
    );
    // Victim's cell left with only terrain.
    assert!(matches!(world.content[0][1], CellContent::Empty));
    // The attacker remains alive.
    assert_eq!(creature_health(&world, attacker), 10);
}

#[test]
fn test_attack_wounds_creature() {
    // AC2: wounded creature does not kill, only reduces life and stays in its cell.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 10);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(3)],
        )
        .expect("attack applies");
    assert_eq!(creature_health(&world, victim), 7, "life must be reduced");
    assert!(
        world.creatures.iter().flatten().any(|c| c.id == victim),
        "victim must remain alive"
    );
    assert!(matches!(world.content[0][1], CellContent::Creature(_)));
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 3)),
        "expected Damaged event, got {:?}",
        r.events
    );
    assert!(
        !r.events.iter().any(|e| matches!(e, Event::Died { .. })),
        "there must be no Died, got {:?}",
        r.events
    );
}

#[test]
fn test_attack_damages_object() {
    // AC3: object reduces its reserve; at 0 disappears at end of tick.
    let (mut world, _, objects) = rock_world();
    // rock at (1,1), reserve 100.
    let attacker = place_creature_with_health(&mut world, 0, 1, 10);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(30)],
        )
        .expect("attack applies");
    assert_eq!(world.objects[0].reserve, 70, "object reserve reduced");
    // The object remains in the cell (disappears at end of tick, not in apply).
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 30)),
        "expected Damaged, got {:?}",
        r.events
    );

    // Lethal damage to the object: reserve reaches 0.
    let _ = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(70)],
        )
        .expect("attack applies");
    assert_eq!(world.objects[0].reserve, 0);
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    let _ = objects;
}

#[test]
fn test_attack_damages_terrain() {
    // AC4: terrain reduces its reserve, including the creature's own cell (0,0).
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 1, 1, 10);

    // Hit adjacent terrain (0,1).
    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(-1), Value::Int(0), Value::Int(40)],
        )
        .expect("attack applies");
    assert_eq!(world.terrains[0].reserve, 60, "terrain reserve reduced");
    assert!(
        r.events.iter().any(|e| matches!(e, Event::Damaged { target, amount, .. } if *target == Pos { x: 0, y: 1 } && *amount == 40)),
        "expected Damaged at (0,1) by 40, got {:?}",
        r.events
    );

    // Hit its own cell (1,1) with dx=0,dy=0: terrain under the creature.
    // All cells share terrain 0, whose reserve dropped to 60; now subtract 60 -> 0.
    let _ = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Int(0), Value::Int(60)],
        )
        .expect("attack applies");
    assert_eq!(
        world.terrains[0].reserve, 0,
        "terrain reserve under creature reduced to 0"
    );
}

#[test]
fn test_attack_out_of_map_noop() {
    // AC5: destination out of map no-op, turn consumed, no error, no event.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(-1), Value::Int(0), Value::Int(100)],
        )
        .expect("attack applies: out of map does not fail");
    assert_eq!(r.sleep_turns, 0);
    assert!(
        r.events.is_empty(),
        "no events must be emitted, got {:?}",
        r.events
    );
    // The world did not change.
    assert_eq!(creature_health(&world, attacker), 10);
}

#[test]
fn test_attack_bad_direction_errors() {
    // AC6: dx or dy outside {-1,0,1} fails naming the argument.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let attack = Attack;

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(2), Value::Int(0), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "dx error: {err:?}");

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Int(-2), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "dy error: {err:?}");
}

#[test]
fn test_attack_text_argument_errors() {
    // AC7: text argument fails naming the argument.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let attack = Attack;

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Text("uno".to_string()), Value::Int(0), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "text dx error: {err:?}");

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Text("dos".to_string()), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "text dy error: {err:?}");

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Int(0),
                Value::Text("tres".to_string()),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("damage"), "text damage error: {err:?}");
}
