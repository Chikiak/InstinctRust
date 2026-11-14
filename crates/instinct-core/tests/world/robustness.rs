//! Robustness tests: test_end_of_tick_min_health_no_panic,
//! test_damage_cell_overflow_and_out_of_map, test_move_creature_invalid_inputs_error,
//! test_world_setters_unknown_id_error.
//! Bodies copied verbatim from tests/world_tests.rs; Spanish comments
//! translated to concise English. Assertion strings untouched.

use super::common::*;

#[test]
fn test_end_of_tick_min_health_no_panic() {
    // Health at i64::MIN: health - 1 overflows; no panic, the creature dies.
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world.place_creature(&species, 0, 0).expect("place");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = i64::MIN;
            c.age = 0;
            c.lifespan = i64::MAX;
        }
    }
    let events = world.end_of_tick_creatures();
    assert!(
        events.iter().any(|e| matches!(e, Event::Died { .. })),
        "the creature with health i64::MIN must die, got {events:?}"
    );
    assert!(
        world.creatures[id.0].is_none(),
        "creature removed from the map"
    );
    assert!(
        world.creatures.iter().all(|c| c.is_none()),
        "the store only has free slots"
    );
}

#[test]
fn test_damage_cell_overflow_and_out_of_map() {
    let mut world = empty_world();
    // Terrain (0,0) with reserve 100. Damage i64::MIN -> 100 - i64::MIN overflows.
    let err = world.damage_cell_content(0, 0, i64::MIN).unwrap_err();
    assert!(
        err.message.contains("overflow"),
        "must report overflow, got {err:?}"
    );
    // The reserve did not change on overflow.
    assert_eq!(world.terrains[0].reserve, 100);

    // Out of the map: no-op, without error.
    let ok = world.damage_cell_content(999, 999, 100);
    assert!(ok.is_ok(), "out of the map is a no-op");
    assert_eq!(world.terrains[0].reserve, 100);
}

#[test]
fn test_move_creature_invalid_inputs_error() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world.place_creature(&species, 1, 1).expect("place");

    // Nonexistent identifier.
    let err = world.move_creature_to(CreatureId(9999), 1, 1).unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "error for unknown id, got {err:?}"
    );

    // Destination out of range: controlled error, no panic.
    let err = world.move_creature_to(id, 999, 999).unwrap_err();
    assert!(
        err.message.contains("outside") || err.message.contains("fuera"),
        "error for destination out of range, got {err:?}"
    );
    // The creature did not move.
    assert_eq!(
        world.creature_position(id).unwrap(),
        instinct_core::event::Pos { x: 1, y: 1 }
    );
}

#[test]
fn test_world_setters_unknown_id_error() {
    let mut world = empty_world();
    let unknown = CreatureId(9999);

    let err = world
        .apply_action_result(
            unknown,
            ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            },
        )
        .unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "apply_action_result unknown id, got {err:?}"
    );

    let err = world.set_creature_health(unknown, 10).unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "set_creature_health unknown id, got {err:?}"
    );

    let err = world
        .set_variable(unknown, "tesoro", Value::Int(1))
        .unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "set_variable unknown id, got {err:?}"
    );
}
