//! Reproduce action tests.

use super::common::*;
use instinct_core::actions::{Catalog, Reproduce};
use instinct_core::event::Pos;
use instinct_core::value::Value;
use instinct_core::world::{CellContent, CreatureState};

#[test]
fn test_catalog_resolves_reproduce() {
    // AC8: `reproduce` resolved with arity 3 and consumes turn.
    let cat = Catalog::default();
    let rep = cat.resolve("reproduce", 3).expect("reproduce arity 3");
    assert_eq!(rep.name(), "reproduce");
    assert_eq!(rep.arity(), 3);
    assert!(rep.consumes_turn());
}

#[test]
fn test_reproduce_child_in_target_cell() {
    // AC1: the child is born in the target cell, the mother stays put, no
    // events and no sleep turns.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);

    let r = apply_reproduce(&mut world, mother, 0, 1, 5);
    assert_eq!(r.sleep_turns, 0);
    assert!(r.events.is_empty());

    let child = world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("child must exist");
    assert_eq!((child.x, child.y), (1, 2));
    assert!(
        matches!(world.content[2][1], CellContent::Creature(cid) if cid == child.id),
        "target cell (1,2) contains the child"
    );
    assert_eq!(world.creature_position(mother).unwrap(), Pos { x: 1, y: 1 });
}

#[test]
fn test_reproduce_child_inherits_traits() {
    // AC1: the child inherits health, age, state, pc, species, faction and
    // program presence from the mother.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);

    let _r = apply_reproduce(&mut world, mother, 0, 1, 5);

    let child = world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("child must exist");
    let mother_creature = world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == mother)
        .expect("mother must exist");
    assert_eq!(child.health, 5);
    assert_eq!(child.age, 0);
    assert_eq!(child.state, CreatureState::Awake);
    assert_eq!(child.pc, 0);
    assert_eq!(child.species.name, "Walker");
    assert_eq!(child.species.faction, mother_creature.species.faction);
    assert_eq!(
        child.species.program.is_some(),
        mother_creature.species.program.is_some()
    );
}

#[test]
fn test_reproduce_child_born_without_variables() {
    // AC3: the child is born without variables.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);

    let _r = apply_reproduce(&mut world, mother, 0, 1, 5);

    let child = world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("child must exist");
    assert!(child.variables.is_empty());
}

#[test]
fn test_reproduce_mother_loses_health() {
    // AC2: the mother loses `health` life when reproducing.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);

    let reproduce = Reproduce;
    let _ = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(1), Value::Int(5)],
        )
        .expect("reproduce applies");
    assert_eq!(creature_health(&world, mother), 5, "mother loses health");
}

#[test]
fn test_reproduce_offspring_no_variables() {
    // AC3: the child does not inherit variables from the mother.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);
    world
        .set_variable(mother, "tesoro", Value::Int(7))
        .expect("set variable");

    let reproduce = Reproduce;
    let _ = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(1), Value::Int(5)],
        )
        .expect("reproduce applies");
    let child = world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("child must exist");
    assert!(
        child.variables.is_empty(),
        "no inherited variables, got {:?}",
        child.variables
    );
    assert_eq!(
        world.variable(mother, "tesoro"),
        Some(Value::Int(7)),
        "the mother keeps its variables"
    );
}

#[test]
fn test_reproduce_abort_occupied_cell() {
    // AC5: cell with creature -> natural abort, no child, mother does not lose life.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);
    let _victim = place_creature_with_health(&mut world, 1, 2, 10); // occupies the target

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(1), Value::Int(5)],
        )
        .expect("reproduce applies: abort is not an error");
    assert_eq!(r.sleep_turns, 0);
    assert!(r.events.is_empty());
    assert_eq!(
        world.creatures.iter().filter(|c| c.is_some()).count(),
        2,
        "only the mother and the victim, no child"
    );
    assert_eq!(
        creature_health(&world, mother),
        10,
        "mother does not lose life when aborting"
    );
}

#[test]
fn test_reproduce_abort_occupied_by_object() {
    // AC5: cell with object -> natural abort, no child.
    let (mut world, _, _) = rock_world(); // rock at (1,1)
    let mother = place_creature_with_health(&mut world, 0, 1, 10);

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(1), Value::Int(0), Value::Int(5)],
        )
        .expect("reproduce applies: abort is not an error");
    assert!(r.events.is_empty());
    assert!(
        !world.creatures.iter().flatten().any(|c| c.id != mother),
        "there must be no child"
    );
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    assert_eq!(creature_health(&world, mother), 10);
}

#[test]
fn test_reproduce_abort_out_of_map() {
    // AC6: destination out of map -> natural abort, no child.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 0, 0, 10);

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(-1), Value::Int(0), Value::Int(5)],
        )
        .expect("reproduce applies: out of map is not an error");
    assert!(r.events.is_empty());
    assert!(
        world.creatures.iter().flatten().all(|c| c.id == mother),
        "there must be no child"
    );
    assert_eq!(creature_health(&world, mother), 10);
}

#[test]
fn test_reproduce_abort_insufficient_health() {
    // AC7: mother life <= health -> natural abort, no child.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 5);

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(1), Value::Int(5)],
        )
        .expect("reproduce applies: abort is not an error");
    assert!(r.events.is_empty());
    assert!(
        !world.creatures.iter().flatten().any(|c| c.id != mother),
        "there must be no child"
    );
    assert_eq!(creature_health(&world, mother), 5);

    // life exactly equal: strict `>` not met.
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(-1), Value::Int(5)],
        )
        .expect("reproduce applies");
    assert!(r.events.is_empty());
    assert!(
        !world.creatures.iter().flatten().any(|c| c.id != mother),
        "there must be no child"
    );
}

#[test]
fn test_reproduce_argument_errors_name_the_argument() {
    // dx / dy outside {-1,0,1} and text arguments name the argument.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);
    let reproduce = Reproduce;

    let err = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(2), Value::Int(0), Value::Int(5)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "dx error: {err:?}");

    let err = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(-2), Value::Int(5)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "dy error: {err:?}");

    let err = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Text("x".to_string()), Value::Int(0), Value::Int(5)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "text dx error: {err:?}");

    let err = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Text("y".to_string()), Value::Int(5)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "text dy error: {err:?}");

    let err = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(0), Value::Text("v".to_string())],
        )
        .unwrap_err();
    assert!(err.message.contains("health"), "text health error: {err:?}");
}
