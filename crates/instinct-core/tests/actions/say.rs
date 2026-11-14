//! Say action tests.

use super::common::*;
use instinct_core::actions::Say;
use instinct_core::event::Event;
use instinct_core::value::Value;

#[test]
fn test_say_emits_event_and_is_free() {
    let say = Say;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);

    assert!(!say.consumes_turn(), "say does not consume a turn");

    let r = say
        .apply(&mut world, id, &[Value::Int(42)])
        .expect("say 42");
    assert_eq!(r.sleep_turns, 0);
    assert_eq!(r.events.len(), 1);
    match &r.events[0] {
        Event::Said {
            tick,
            species,
            pos,
            text,
        } => {
            assert_eq!(*tick, 0);
            assert_eq!(species, "Walker");
            assert_eq!(*pos, instinct_core::event::Pos { x: 1, y: 1 });
            assert_eq!(text, "42");
        }
        _ => panic!("expected Said event"),
    }

    // Text argument is spoken as-is.
    let r = say
        .apply(&mut world, id, &[Value::Text("hola".to_string())])
        .expect("say text");
    match &r.events[0] {
        Event::Said { text, .. } => assert_eq!(text, "hola"),
        _ => panic!("expected Said event"),
    }
}
