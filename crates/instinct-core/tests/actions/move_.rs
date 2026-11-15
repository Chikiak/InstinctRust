//! Move action tests.

use super::common::*;
use instinct_core::actions::Move;
use instinct_core::value::Value;
use std::rc::Rc;

#[test]
fn test_move_right_two_ends_at_2_0() {
    // (0,0) dx=1 dy=0 speed=2 -> (1,0) free, (2,0) free -> ends at (2,0).
    let mv = Move;
    let (mut world, _, _) = rock_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 0, 0)
        .unwrap();
    let r = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(1), Value::Int(0), Value::Int(2)],
        )
        .unwrap();
    assert_eq!(r.sleep_turns, 0);
    let pos = world.creature_position(id).unwrap();
    assert_eq!((pos.x, pos.y), (2, 0));
    assert!(
        matches!(world.content[0][2], CellContent::Creature(cid) if cid == id),
        "cell (2,0) holds the creature"
    );
}

#[test]
fn test_move_immediate_block_stays_at_0_0() {
    // (0,0) dx=1 dy=1 speed=5 -> (1,1) has rock -> stays at (0,0).
    let mv = Move;
    let (mut world, _, _) = rock_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 0, 0)
        .unwrap();
    let r = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(1), Value::Int(1), Value::Int(5)],
        )
        .unwrap();
    assert_eq!(r.sleep_turns, 0);
    let pos = world.creature_position(id).unwrap();
    assert_eq!((pos.x, pos.y), (0, 0));
    assert!(
        matches!(world.content[0][0], CellContent::Creature(cid) if cid == id),
        "cell (0,0) holds the creature"
    );
}

#[test]
fn test_move_speed_greater_than_free_path_ends_at_4_0() {
    // (0,0) dx=1 dy=0 speed=10 -> row 0 all free -> ends at (4,0).
    let mv = Move;
    let (mut world, _, _) = rock_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 0, 0)
        .unwrap();
    let r = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(1), Value::Int(0), Value::Int(10)],
        )
        .unwrap();
    assert_eq!(r.sleep_turns, 0);
    let pos = world.creature_position(id).unwrap();
    assert_eq!((pos.x, pos.y), (4, 0));
    assert!(
        matches!(world.content[0][4], CellContent::Creature(cid) if cid == id),
        "cell (4,0) holds the creature"
    );
}

#[test]
fn test_move_down_two_ends_at_0_2() {
    // (0,0) dx=0 dy=1 speed=2 -> (0,1) free, (0,2) free -> ends at (0,2).
    let mv = Move;
    let (mut world, _, _) = rock_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 0, 0)
        .unwrap();
    let r = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(0), Value::Int(1), Value::Int(2)],
        )
        .unwrap();
    assert_eq!(r.sleep_turns, 0);
    let pos = world.creature_position(id).unwrap();
    assert_eq!((pos.x, pos.y), (0, 2));
    assert!(
        matches!(world.content[2][0], CellContent::Creature(cid) if cid == id),
        "cell (0,2) holds the creature"
    );
}

#[test]
fn test_move_up_two_ends_at_2_0() {
    // (2,2) dx=0 dy=-1 speed=2 -> (2,1) free, (2,0) free -> ends at (2,0).
    let mv = Move;
    let (mut world, _, _) = rock_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 2, 2)
        .unwrap();
    let r = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(0), Value::Int(-1), Value::Int(2)],
        )
        .unwrap();
    assert_eq!(r.sleep_turns, 0);
    let pos = world.creature_position(id).unwrap();
    assert_eq!((pos.x, pos.y), (2, 0));
    assert!(
        matches!(world.content[0][2], CellContent::Creature(cid) if cid == id),
        "cell (2,0) holds the creature"
    );
}

#[test]
fn test_move_zero_direction_or_speed_does_not_move() {
    let mv = Move;
    let cases: Vec<(i64, i64, i64, &'static str)> = vec![
        (0, 0, 3, "zero_direction"),
        (1, 0, 0, "zero_speed"),
        (1, 0, -2, "negative_speed"),
    ];

    for (dx, dy, speed, desc) in cases {
        let (mut world, _, _) = rock_world();
        let id = world
            .place_creature(&Rc::new(Species::new("Walker")), 0, 0)
            .unwrap_or_else(|e| panic!("{desc}: place: {e:?}"));
        let r = mv
            .apply(
                &mut world,
                id,
                &[Value::Int(dx), Value::Int(dy), Value::Int(speed)],
            )
            .unwrap_or_else(|e| panic!("{desc}: apply: {e:?}"));
        assert_eq!(r.sleep_turns, 0, "{desc}: sleep_turns");
        let pos = world.creature_position(id).unwrap();
        assert_eq!((pos.x, pos.y), (0, 0), "{desc}: no move");
    }
}

#[test]
fn test_move_argument_errors_name_the_argument() {
    let mv = Move;
    let (mut world, _, _) = rock_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 0, 0)
        .expect("place walker at free cell");

    // dx outside {-1,0,1}.
    let err = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(2), Value::Int(0), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "dx error: {err:?}");

    // dy outside {-1,0,1}.
    let err = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(0), Value::Int(-2), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "dy error: {err:?}");

    // Text argument for dx.
    let err = mv
        .apply(
            &mut world,
            id,
            &[Value::Text("x".to_string()), Value::Int(0), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "text dx error: {err:?}");

    // Text argument for speed.
    let err = mv
        .apply(
            &mut world,
            id,
            &[Value::Int(1), Value::Int(0), Value::Text("s".to_string())],
        )
        .unwrap_err();
    assert!(err.message.contains("speed"), "text speed error: {err:?}");
}
