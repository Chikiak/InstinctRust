//! Unit tests for the `event` module.

use instinct_core::event::{Event, Pos, WorldOutput};

/// Compartte un Vec de eventos entre el puerto de salida y el test mediante
/// Rc<RefCell>, para poder comprobar que el puerto recibió el evento.
struct SharedOutput {
    events: std::rc::Rc<std::cell::RefCell<Vec<Event>>>,
}

impl WorldOutput for SharedOutput {
    fn emit(&mut self, event: &Event) {
        self.events.borrow_mut().push(event.clone());
    }
}

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

// ---------------------------------------------------------------------------
// AC2: WorldOutput - el puerto de salida recibe los eventos y drain_events sigue
// ---------------------------------------------------------------------------

#[test]
fn test_world_output_forwards_to_bus() {
    use instinct_core::resources::{Map, Object, Terrain};
    use instinct_core::world::World;
    use std::cell::RefCell;
    use std::rc::Rc;

    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 3], vec!['.'; 3]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 1,
        reserve: 100,
    }];
    let objects = vec![Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 10,
        reserve: 10,
    }];

    // Sin puerto: emit_event solo guarda en events.
    let mut world = World::from_map(&map, &terrains, &objects);
    let ev = Event::Said {
        tick: 1,
        species: "Sp".to_string(),
        pos: Pos { x: 0, y: 0 },
        text: "hola".to_string(),
    };
    world.emit_event(ev.clone());
    assert_eq!(world.events.len(), 1);
    assert_eq!(world.drain_events(), vec![ev.clone()]);
    assert!(world.events.is_empty());

    // Con puerto: el puerto recibe el evento y drain_events sigue funcionando.
    let captured = Rc::new(RefCell::new(Vec::<Event>::new()));
    let captured_clone = Rc::clone(&captured);
    let mut world = World::from_map(&map, &terrains, &objects);
    world.set_output(Box::new(SharedOutput {
        events: captured_clone,
    }));

    world.emit_event(ev.clone());
    let drained = world.drain_events();
    assert_eq!(drained, vec![ev.clone()]);
    assert!(world.events.is_empty());
    // El puerto recibió el mismo evento.
    assert_eq!(captured.borrow().len(), 1);
    assert_eq!(captured.borrow()[0], ev);
}
