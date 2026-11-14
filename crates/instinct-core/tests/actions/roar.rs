//! Roar action tests.

use super::common::*;
use instinct_core::actions::Roar;
use instinct_core::event::{Event, Pos};
use instinct_core::value::Value;

#[test]
fn test_catalog_resolves_roar() {
    // AC5: `roar` resolved with arity 1 and consumes turn.
    let cat = Catalog::default();
    let roar = cat.resolve("roar", 1).expect("roar arity 1");
    assert_eq!(roar.name(), "roar");
    assert_eq!(roar.arity(), 1);
    assert!(roar.consumes_turn());
}

#[test]
fn test_roar_emits_event() {
    // AC1: `roar("text")` emits Roared with tick, species, position and text.
    let roar = Roar;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);

    assert!(roar.consumes_turn(), "AC3: roar consumes turn");

    let r = roar
        .apply(&mut world, id, &[Value::Text("¡auau!".to_string())])
        .expect("roar applies");
    assert_eq!(r.sleep_turns, 0);
    assert_eq!(r.events.len(), 1);
    match &r.events[0] {
        Event::Roared {
            tick,
            species,
            pos,
            text,
        } => {
            assert_eq!(*tick, world.ticks, "roar tick");
            assert_eq!(species, "Walker");
            assert_eq!(*pos, Pos { x: 1, y: 1 });
            assert_eq!(text, "¡auau!");
        }
        _ => panic!("expected Roared event, got {:?}", r.events),
    }
}

#[test]
fn test_roar_consumes_turn() {
    // AC3: `roar` consumes turn even if the message is empty.
    let roar = Roar;
    assert!(roar.consumes_turn());
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);
    let r = roar
        .apply(&mut world, id, &[Value::Text("".to_string())])
        .expect("empty roar applies");
    assert_eq!(r.sleep_turns, 0);
    assert_eq!(r.events.len(), 1);
    assert_eq!(world.roars_current.len(), 1);
}

#[test]
fn test_roar_number_as_text() {
    // AC2: `roar(42)` shows "42" as text.
    let roar = Roar;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);

    let r = roar
        .apply(&mut world, id, &[Value::Int(42)])
        .expect("roar 42 applies");
    match &r.events[0] {
        Event::Roared { text, .. } => assert_eq!(text, "42"),
        _ => panic!("expected Roared event, got {:?}", r.events),
    }
}

#[test]
fn test_roar_stored_with_tick() {
    // AC4: the roar is stored associated with the tick it was emitted.
    let roar = Roar;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);

    let _ = roar
        .apply(&mut world, id, &[Value::Text("hola".to_string())])
        .expect("roar applies");

    assert_eq!(world.roars_current.len(), 1, "stored in current tick");
    assert!(
        world.roars_previous.is_empty(),
        "nothing from previous tick yet"
    );
    let stored = &world.roars_current[0];
    assert_eq!(stored.tick, world.ticks, "emission tick");
    assert_eq!(stored.species, "Walker");
    assert_eq!(stored.pos, Pos { x: 1, y: 1 });
    assert_eq!(stored.text, "hola");

    // An empty roar is also stored and consumes turn.
    let _ = roar
        .apply(&mut world, id, &[Value::Text("".to_string())])
        .expect("empty roar applies");
    assert_eq!(world.roars_current.len(), 2);
}
