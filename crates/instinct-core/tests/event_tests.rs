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

// ---------------------------------------------------------------------------
// AC5: Display de cada variante de Event (fijado, no modificado)
// ---------------------------------------------------------------------------

#[test]
fn test_display_said() {
    let ev = Event::Said {
        tick: 3,
        species: "Walker".to_string(),
        pos: Pos { x: 5, y: 2 },
        text: "hola".to_string(),
    };
    assert_eq!(ev.to_string(), "[tick 3] Walker@5,2 dice: hola");
}

#[test]
fn test_display_died() {
    let ev = Event::Died {
        tick: 7,
        species: "Slug".to_string(),
        pos: Pos { x: 1, y: 0 },
        cause: "edad".to_string(),
    };
    assert_eq!(ev.to_string(), "[tick 7] Slug@1,0 muere por edad");
}

#[test]
fn test_display_runtime_error() {
    let ev = Event::RuntimeError {
        tick: 2,
        species: "Sp".to_string(),
        pos: Pos { x: 0, y: 0 },
        message: "division by zero".to_string(),
        line: 4,
    };
    assert_eq!(
        ev.to_string(),
        "[tick 2] Sp@0,0 error en línea 4: division by zero"
    );
}

#[test]
fn test_display_damaged() {
    let ev = Event::Damaged {
        tick: 1,
        species: "A".to_string(),
        pos: Pos { x: 1, y: 1 },
        target: Pos { x: 2, y: 3 },
        amount: 5,
    };
    assert_eq!(ev.to_string(), "[tick 1] A@1,1 daña 2,3 por 5");
}

#[test]
fn test_display_roared() {
    let ev = Event::Roared {
        tick: 9,
        species: "R".to_string(),
        pos: Pos { x: 0, y: 0 },
        text: "aaa".to_string(),
    };
    assert_eq!(ev.to_string(), "[tick 9] R@0,0 ruge: aaa");
}
