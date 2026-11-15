//! Memento tests: test_creature_memento_captures_state,
//! test_restore_creature_from_memento, test_restore_creature_from_memento_unknown_id.
//! Bodies copied verbatim from tests/world_tests.rs; Spanish comments
//! translated to concise English. Assertion strings untouched.

use super::common::*;

#[test]
fn test_creature_memento_captures_state() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world
        .place_creature(&species, 2, 2)
        .expect("place creature");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = 42;
            c.age = 7;
            c.pc = 5;
            c.state = CreatureState::Sleeping(3);
            c.variables.insert("oro".to_string(), Value::Int(100));
            c.variables.insert("vida_extra".to_string(), Value::Int(10));
        }
    }

    let memento = world.creatures[id.0].as_ref().expect("creature").memento();

    assert_eq!(memento.id, id);
    assert_eq!(memento.health, 42);
    assert_eq!(memento.age, 7);
    assert_eq!(memento.x, 2);
    assert_eq!(memento.y, 2);
    assert_eq!(memento.pc, 5);
    assert_eq!(memento.state, CreatureState::Sleeping(3));
    assert_eq!(memento.variables.get("oro"), Some(&Value::Int(100)));
    assert_eq!(memento.variables.get("vida_extra"), Some(&Value::Int(10)));
}

#[test]
fn test_restore_creature_from_memento() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world
        .place_creature(&species, 1, 1)
        .expect("place creature");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = 10;
            c.age = 0;
            c.pc = 0;
            c.state = CreatureState::Awake;
        }
    }
    let memento = world.creatures[id.0].as_ref().expect("creature").memento();

    // Mutate the creature's state.
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = 1;
            c.age = 99;
            c.x = 9;
            c.y = 9;
            c.pc = 99;
            c.state = CreatureState::Sleeping(5);
            c.variables.insert("basura".to_string(), Value::Int(-1));
        }
    }

    let restored = world.restore_creature_from_memento(&memento);
    assert!(restored, "restoration must succeed");

    let c = world.creatures[id.0].as_ref().expect("creature");
    assert_eq!(c.health, 10);
    assert_eq!(c.age, 0);
    assert_eq!((c.x, c.y), (1, 1));
    assert_eq!(c.pc, 0);
    assert_eq!(c.state, CreatureState::Awake);
    assert!(c.variables.is_empty(), "variables must be restored empty");
}

#[test]
fn test_restore_creature_from_memento_unknown_id() {
    let mut world = empty_world();
    let memento = CreatureMemento {
        id: CreatureId(9999),
        health: 0,
        age: 0,
        x: 0,
        y: 0,
        pc: 0,
        state: CreatureState::Awake,
        variables: HashMap::new(),
    };
    assert!(
        !world.restore_creature_from_memento(&memento),
        "restoring unknown id must fail without panic"
    );
}
