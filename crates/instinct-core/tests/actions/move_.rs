//! Move action tests.

use super::common::*;
use instinct_core::actions::Move;
use instinct_core::value::Value;
use std::rc::Rc;

#[test]
fn test_move_stops_before_obstacles() {
    let mv = Move;
    struct MoveCase {
        sx: usize,
        sy: usize,
        dx: i64,
        dy: i64,
        speed: i64,
        ex: usize,
        ey: usize,
        desc: &'static str,
    }
    let cases: Vec<MoveCase> = vec![
        // Right 2 from (0,0): (1,0) free, (2,0) free -> ends at (2,0).
        MoveCase {
            sx: 0,
            sy: 0,
            dx: 1,
            dy: 0,
            speed: 2,
            ex: 2,
            ey: 0,
            desc: "right",
        },
        // Diagonal down-right from (0,0): (1,1) has rock -> stops at (0,0).
        MoveCase {
            sx: 0,
            sy: 0,
            dx: 1,
            dy: 1,
            speed: 5,
            ex: 0,
            ey: 0,
            desc: "immediate_block",
        },
        // Right from (0,0) speed 10: row 0 is all free -> ends at (4,0).
        MoveCase {
            sx: 0,
            sy: 0,
            dx: 1,
            dy: 0,
            speed: 10,
            ex: 4,
            ey: 0,
            desc: "speed_greater_than_free_path",
        },
        // Down from (0,0): (0,1) free, (0,2) free -> ends at (0,2).
        MoveCase {
            sx: 0,
            sy: 0,
            dx: 0,
            dy: 1,
            speed: 2,
            ex: 0,
            ey: 2,
            desc: "down",
        },
        // Up from (2,2): (2,1) free, (2,0) free -> ends at (2,0).
        MoveCase {
            sx: 2,
            sy: 2,
            dx: 0,
            dy: -1,
            speed: 2,
            ex: 2,
            ey: 0,
            desc: "up",
        },
    ];

    for case in cases {
        let (mut world, terrains, objects) = rock_world();
        let id = world
            .place_creature(&Rc::new(Species::new("Walker")), case.sx, case.sy)
            .unwrap_or_else(|e| panic!("{}: place: {e:?}", case.desc));
        let r = mv
            .apply(
                &mut world,
                id,
                &[
                    Value::Int(case.dx),
                    Value::Int(case.dy),
                    Value::Int(case.speed),
                ],
            )
            .unwrap_or_else(|e| panic!("{}: apply: {e:?}", case.desc));
        assert_eq!(r.sleep_turns, 0, "{} sleep_turns", case.desc);
        let pos = world
            .creature_position(id)
            .unwrap_or_else(|e| panic!("{}: position: {e:?}", case.desc));
        assert_eq!(
            (pos.x, pos.y),
            (case.ex, case.ey),
            "{} expected ({},{}) got ({},{})",
            case.desc,
            case.ex,
            case.ey,
            pos.x,
            pos.y
        );
        // Grid occupancy consistent.
        match &world.content[case.ey][case.ex] {
            instinct_core::world::CellContent::Creature(cid) => assert_eq!(*cid, id),
            other => panic!(
                "{}: cell should hold the creature, got {other:?}",
                case.desc
            ),
        }
        let _ = (&terrains, &objects);
    }
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
