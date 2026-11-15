use instinct_core::event::{Event, Pos};

// ---------------------------------------------------------------------------
// AC5: Display of each Event variant (frozen, not modified)
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
        cause: "age".to_string(),
    };
    assert_eq!(ev.to_string(), "[tick 7] Slug@1,0 dies of age");
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
