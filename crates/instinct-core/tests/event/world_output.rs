use super::shared::SharedOutput;
use instinct_core::event::{Event, Pos};

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

    // No port: emit_event only stores in events.
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

    // With port: the port receives the event and drain_events keeps working.
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
    // The port received the same event.
    assert_eq!(captured.borrow().len(), 1);
    assert_eq!(captured.borrow()[0], ev);
}
