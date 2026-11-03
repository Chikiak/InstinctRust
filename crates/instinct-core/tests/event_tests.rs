//! Unit tests for the `event` module.

use instinct_core::event::{Event, Pos};

#[test]
fn test_said_event_fields() {
    let ev = Event::Said {
        tick: 3,
        species: "Walker".to_string(),
        pos: Pos { x: 5, y: 2 },
        text: "hola".to_string(),
    };
    match ev {
        Event::Said {
            tick,
            species,
            pos,
            text,
        } => {
            assert_eq!(tick, 3);
            assert_eq!(species, "Walker");
            assert_eq!(pos.x, 5);
            assert_eq!(pos.y, 2);
            assert_eq!(text, "hola");
        }
        _ => panic!("expected Said event"),
    }
}

#[test]
fn test_event_clone_and_eq() {
    let a = Event::Said {
        tick: 0,
        species: "Sp".to_string(),
        pos: Pos { x: 0, y: 0 },
        text: "x".to_string(),
    };
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(
        a,
        Event::Said {
            tick: 1,
            species: "Sp".to_string(),
            pos: Pos { x: 0, y: 0 },
            text: "x".to_string(),
        }
    );
}
