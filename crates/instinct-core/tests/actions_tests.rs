//! Unit tests for the `actions` module: the `Action` trait, the `Catalog`,
//! and the `wait`, `move`, `say` actions.

use instinct_core::actions::{
    Action, ActionResult, Attack, Catalog, Consume, Move, Reproduce, Say, Wait,
};
use instinct_core::diagnostic::Diagnostic;
use instinct_core::event::{Event, Pos};
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::{CellContent, CreatureId, World};
use std::rc::Rc;

fn world_with_grass() -> (World, Vec<Terrain>, Vec<Object>) {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 1,
        reserve: 100,
    }];
    let objects = vec![];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    (world, terrains, objects)
}

fn place_walker(world: &mut World) -> CreatureId {
    world
        .place_creature(Species::new("Walker"), 1, 1)
        .expect("place creature")
}

// ---------------------------------------------------------------------------
// Catalog (AC7)
// ---------------------------------------------------------------------------

#[test]
fn test_catalog_resolves_three_actions_with_arity() {
    let cat = Catalog::default();
    let wait = cat.resolve("wait", 1).expect("wait arity 1");
    assert_eq!(wait.name(), "wait");
    assert_eq!(wait.arity(), 1);
    assert!(wait.consumes_turn());

    let mv = cat.resolve("move", 3).expect("move arity 3");
    assert_eq!(mv.name(), "move");
    assert_eq!(mv.arity(), 3);
    assert!(mv.consumes_turn());

    let say = cat.resolve("say", 1).expect("say arity 1");
    assert_eq!(say.name(), "say");
    assert_eq!(say.arity(), 1);
    assert!(!say.consumes_turn());
}

#[test]
fn test_catalog_rejects_unknown_action_and_wrong_arity() {
    let cat = Catalog::default();
    assert!(cat.resolve("nope", 0).is_err());
    assert!(cat.resolve("wait", 2).is_err());
    assert!(cat.resolve("move", 2).is_err());
    assert!(cat.resolve("say", 0).is_err());
}

#[test]
fn test_catalog_extension_with_test_only_action() {
    // A test-defined action type not in the existing sources.
    #[derive(Debug)]
    struct TestAction;

    impl Action for TestAction {
        fn name(&self) -> &str {
            "test_action"
        }
        fn arity(&self) -> usize {
            0
        }
        fn consumes_turn(&self) -> bool {
            false
        }
        fn apply(
            &self,
            _world: &mut World,
            _creature_id: CreatureId,
            _args: &[Value],
        ) -> Result<ActionResult, Diagnostic> {
            Ok(ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            })
        }
    }

    let mut cat = Catalog::default();
    assert!(cat.resolve("test_action", 0).is_err());
    assert!(
        cat.register(Rc::new(TestAction)).is_ok(),
        "register the test action"
    );
    let ta = cat.resolve("test_action", 0).expect("resolve test_action");
    assert_eq!(ta.name(), "test_action");
    assert_eq!(ta.arity(), 0);
    // Existing actions still resolve.
    assert!(cat.resolve("wait", 1).is_ok());
    assert!(cat.resolve("move", 3).is_ok());
    assert!(cat.resolve("say", 1).is_ok());
}

// ---------------------------------------------------------------------------
// wait (AC5)
// ---------------------------------------------------------------------------

#[test]
fn test_wait_sleep_turns() {
    let wait = Wait;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);

    // wait 3 -> sleep 2 turns.
    let r = wait
        .apply(&mut world, id, &[Value::Int(3)])
        .expect("wait 3");
    assert_eq!(r.sleep_turns, 2);
    assert!(r.events.is_empty());
    assert!(wait.consumes_turn());

    // wait 1 -> sleep 0.
    let r = wait
        .apply(&mut world, id, &[Value::Int(1)])
        .expect("wait 1");
    assert_eq!(r.sleep_turns, 0);

    // wait 0 (and negative) counts as 1 -> sleep 0.
    let r = wait
        .apply(&mut world, id, &[Value::Int(0)])
        .expect("wait 0");
    assert_eq!(r.sleep_turns, 0);
    let r = wait
        .apply(&mut world, id, &[Value::Int(-5)])
        .expect("wait -5");
    assert_eq!(r.sleep_turns, 0);
}

#[test]
fn test_wait_text_argument_fails() {
    let wait = Wait;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);
    let err = wait
        .apply(&mut world, id, &[Value::Text("tres".to_string())])
        .unwrap_err();
    assert!(
        err.message.contains("n"),
        "error must name argument 'n', got: {err:?}"
    );
    assert!(err.message.contains("text"));
}

// ---------------------------------------------------------------------------
// say (AC6)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// move (AC3, AC4)
// ---------------------------------------------------------------------------

fn rock_world() -> (World, Vec<Terrain>, Vec<Object>) {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.', '#', '.', '.', '.'], vec!['.'; 5]],
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
        resource_max: 100,
        reserve: 100,
    }];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    (world, terrains, objects)
}

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
            desc: "derecha",
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
            desc: "bloqueo inmediato",
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
            desc: "velocidad mayor que recorrido libre",
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
            desc: "hacia abajo",
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
            desc: "hacia arriba",
        },
    ];

    for case in cases {
        let (mut world, terrains, objects) = rock_world();
        let id = world
            .place_creature(Species::new("Walker"), case.sx, case.sy)
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
        (0, 0, 3, "direccion cero"),
        (1, 0, 0, "speed cero"),
        (1, 0, -2, "speed negativo"),
    ];

    for (dx, dy, speed, desc) in cases {
        let (mut world, _, _) = rock_world();
        let id = world
            .place_creature(Species::new("Walker"), 0, 0)
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
    world
        .place_creature(Species::new("Walker"), 0, 0)
        .expect("place walker at free cell");
    let id = world.creatures[0].id;

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

// ---------------------------------------------------------------------------
// attack (AC1-AC8)
// ---------------------------------------------------------------------------

/// Coloca una criatura y le fija la salud (Species::new tiene health 0).
fn place_creature_with_health(world: &mut World, x: usize, y: usize, health: i64) -> CreatureId {
    let id = world
        .place_creature(Species::new("Walker"), x, y)
        .expect("place creature");
    for c in world.creatures.iter_mut() {
        if c.id == id {
            c.health = health;
        }
    }
    id
}

fn creature_health(world: &World, id: CreatureId) -> i64 {
    world
        .creatures
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.health)
        .expect("creature exists")
}

#[test]
fn test_catalog_resolves_attack() {
    let cat = Catalog::default();
    let atk = cat.resolve("attack", 3).expect("attack arity 3");
    assert_eq!(atk.name(), "attack");
    assert_eq!(atk.arity(), 3);
    assert!(atk.consumes_turn());
}

#[test]
fn test_attack_kills_creature_immediately() {
    // AC1: criatura muere al golpe, casilla queda solo terreno, Died causa vida este tick.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 5);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(10)],
        )
        .expect("attack applies");
    assert_eq!(r.sleep_turns, 0);
    assert!(
        r.events.len() >= 2,
        "esperaba Damaged + Died, got {:?}",
        r.events
    );

    // Damaged evento.
    assert!(r
        .events
        .iter()
        .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 10)));
    // Died causa vida.
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Died { cause, .. } if cause == "vida")),
        "esperaba Died causa vida, got {:?}",
        r.events
    );
    // Víctima removida.
    assert!(
        world.creatures.iter().all(|c| c.id != victim),
        "víctima debe estar removida"
    );
    // Casilla de la víctima queda solo terreno.
    assert!(matches!(world.content[0][1], CellContent::Empty));
    // El atacante sigue vivo.
    assert_eq!(creature_health(&world, attacker), 10);
}

#[test]
fn test_attack_wounds_creature() {
    // AC2: criatura herida no mata, solo reduce vida y sigue en su casilla.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 10);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(3)],
        )
        .expect("attack applies");
    assert_eq!(creature_health(&world, victim), 7, "vida debe reducirse");
    assert!(
        world.creatures.iter().any(|c| c.id == victim),
        "víctima debe seguir viva"
    );
    assert!(matches!(world.content[0][1], CellContent::Creature(_)));
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 3)),
        "esperaba Damaged evento, got {:?}",
        r.events
    );
    assert!(
        !r.events.iter().any(|e| matches!(e, Event::Died { .. })),
        "no debe haber Died, got {:?}",
        r.events
    );
}

#[test]
fn test_attack_damages_object() {
    // AC3: objeto reduce su reserva; al 0 desaparece al final del tick.
    let (mut world, _, objects) = rock_world();
    // roca en (1,1), reserve 100.
    let attacker = place_creature_with_health(&mut world, 0, 1, 10);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(30)],
        )
        .expect("attack applies");
    assert_eq!(world.objects[0].reserve, 70, "reserva del objeto reducida");
    // El objeto sigue en la celda (desaparece al final del tick, no en apply).
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 30)),
        "esperaba Damaged, got {:?}",
        r.events
    );

    // Daño letal al objeto: reserve llega a 0.
    let _ = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(70)],
        )
        .expect("attack applies");
    assert_eq!(world.objects[0].reserve, 0);
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    let _ = objects;
}

#[test]
fn test_attack_damages_terrain() {
    // AC4: terreno reduce su reserva, incluida la casilla propia (0,0).
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 1, 1, 10);

    // Golpear terreno adyacente (0,1).
    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(-1), Value::Int(0), Value::Int(40)],
        )
        .expect("attack applies");
    assert_eq!(
        world.terrains[0].reserve, 60,
        "reserva del terreno reducida"
    );
    assert!(
        r.events.iter().any(|e| matches!(e, Event::Damaged { target, amount, .. } if *target == Pos { x: 0, y: 1 } && *amount == 40)),
        "esperaba Damaged en (0,1) por 40, got {:?}",
        r.events
    );

    // Golpear la propia casilla (1,1) con dx=0,dy=0: terreno bajo la criatura.
    // Todas las celdas comparten el terreno 0, cuya reserva bajó a 60; ahora se le restan 60 -> 0.
    let _ = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Int(0), Value::Int(60)],
        )
        .expect("attack applies");
    assert_eq!(
        world.terrains[0].reserve, 0,
        "reserva del terreno bajo la criatura reducida a 0"
    );
}

#[test]
fn test_attack_out_of_map_noop() {
    // AC5: destino fuera del mapa no-op, turno consumido, sin error, sin evento.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);

    let attack = Attack;
    let r = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(-1), Value::Int(0), Value::Int(100)],
        )
        .expect("attack applies: fuera del mapa no falla");
    assert_eq!(r.sleep_turns, 0);
    assert!(
        r.events.is_empty(),
        "no debe emitir eventos, got {:?}",
        r.events
    );
    // El mundo no cambió.
    assert_eq!(creature_health(&world, attacker), 10);
}

#[test]
fn test_attack_bad_direction_errors() {
    // AC6: dx o dy fuera de {-1,0,1} falla nombrando el argumento.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let attack = Attack;

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(2), Value::Int(0), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "dx error: {err:?}");

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Int(-2), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "dy error: {err:?}");
}

#[test]
fn test_attack_text_argument_errors() {
    // AC7: argumento de texto falla nombrando el argumento.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let attack = Attack;

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Text("uno".to_string()), Value::Int(0), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "text dx error: {err:?}");

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Text("dos".to_string()), Value::Int(1)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "text dy error: {err:?}");

    let err = attack
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Int(0),
                Value::Text("tres".to_string()),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("damage"), "text damage error: {err:?}");
}

// ---------------------------------------------------------------------------
// consume (AC1-AC8)
// ---------------------------------------------------------------------------

#[test]
fn test_catalog_resolves_consume() {
    let cat = Catalog::default();
    let cons = cat.resolve("consume", 4).expect("consume arity 4");
    assert_eq!(cons.name(), "consume");
    assert_eq!(cons.arity(), 4);
    assert!(cons.consumes_turn());
}

#[test]
fn test_consume_damages_creature() {
    // AC1: consume contra criatura reduce su vida; <=0 muere en el acto.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 5);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(10), Value::Int(3)],
        )
        .expect("consume applies");
    assert_eq!(r.sleep_turns, 0);
    assert!(
        r.events
            .iter()
            .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 10)),
        "esperaba Damaged, got {:?}",
        r.events
    );
    assert!(
        world.creatures.iter().all(|c| c.id != victim),
        "víctima debe estar removida"
    );
    assert!(matches!(world.content[0][1], CellContent::Empty));
    // Atacante curado en 3 (sin tope).
    assert_eq!(creature_health(&world, attacker), 13, "atacante curado");
}

#[test]
fn test_consume_heals_attacker() {
    // AC2: cura a la atacante en heal, sin tope.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let victim = place_creature_with_health(&mut world, 1, 0, 10);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(3), Value::Int(7)],
        )
        .expect("consume applies");
    assert_eq!(creature_health(&world, victim), 7);
    assert_eq!(
        creature_health(&world, attacker),
        17,
        "atacante +7 sin tope"
    );
    assert!(r
        .events
        .iter()
        .any(|e| matches!(e, Event::Damaged { amount, .. } if *amount == 3)),);
}

#[test]
fn test_consume_damages_object() {
    // AC3: objeto reduce su reserva; al 0 desaparece al final del tick.
    let (mut world, _, _) = rock_world();
    let attacker = place_creature_with_health(&mut world, 0, 1, 10);

    let consume = Consume;
    let _ = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(30), Value::Int(5)],
        )
        .expect("consume applies");
    assert_eq!(world.objects[0].reserve, 70);
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    assert!(
        creature_health(&world, attacker) == 15,
        "atacante curado +5"
    );
}

#[test]
fn test_consume_damages_terrain() {
    // AC4: terreno reduce su reserva, incluida (0,0) bajo la propia criatura.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 1, 1, 10);

    let consume = Consume;
    let _ = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(-1),
                Value::Int(0),
                Value::Int(40),
                Value::Int(10),
            ],
        )
        .expect("consume applies");
    assert_eq!(world.terrains[0].reserve, 60);
    assert_eq!(creature_health(&world, attacker), 20);
}

#[test]
fn test_consume_out_of_map_noop() {
    // AC5: fuera del mapa no-op, ni daño ni curación, turno consumido.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(-1),
                Value::Int(0),
                Value::Int(100),
                Value::Int(50),
            ],
        )
        .expect("consume applies");
    assert_eq!(r.sleep_turns, 0);
    assert!(r.events.is_empty(), "sin eventos, got {:?}", r.events);
    assert_eq!(
        creature_health(&world, attacker),
        10,
        "sin curación fuera del mapa"
    );
}

#[test]
fn test_consume_bad_direction_errors() {
    // AC6: dx/dy fuera de {-1,0,1} error nombrando el argumento.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let consume = Consume;

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(2), Value::Int(0), Value::Int(1), Value::Int(0)],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "dx error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(0), Value::Int(-2), Value::Int(1), Value::Int(0)],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "dy error: {err:?}");
}

#[test]
fn test_consume_text_argument_errors() {
    // AC7: argumento de texto error nombrando el argumento.
    let (mut world, _, _) = world_with_grass();
    let attacker = place_creature_with_health(&mut world, 0, 0, 10);
    let consume = Consume;

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Text("uno".to_string()),
                Value::Int(0),
                Value::Int(1),
                Value::Int(0),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("dx"), "text dx error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Text("dos".to_string()),
                Value::Int(1),
                Value::Int(0),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("dy"), "text dy error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Int(0),
                Value::Text("tres".to_string()),
                Value::Int(0),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("damage"), "text damage error: {err:?}");

    let err = consume
        .apply(
            &mut world,
            attacker,
            &[
                Value::Int(0),
                Value::Int(0),
                Value::Int(1),
                Value::Text("cuatro".to_string()),
            ],
        )
        .unwrap_err();
    assert!(err.message.contains("heal"), "text heal error: {err:?}");
}

// ---------------------------------------------------------------------------
// reproduce (AC1-AC8)
// ---------------------------------------------------------------------------

#[test]
fn test_catalog_resolves_reproduce() {
    // AC8: `reproduce` resuelto con aridad 3 y consume turno.
    let cat = Catalog::default();
    let rep = cat.resolve("reproduce", 3).expect("reproduce arity 3");
    assert_eq!(rep.name(), "reproduce");
    assert_eq!(rep.arity(), 3);
    assert!(rep.consumes_turn());
}

#[test]
fn test_reproduce_creates_offspring() {
    // AC1: casilla solo terreno y madre con vida > health -> cría con
    // vida = health, edad 0, sin variables y pc al inicio.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(1), Value::Int(5)],
        )
        .expect("reproduce applies");
    assert_eq!(r.sleep_turns, 0);
    assert!(r.events.is_empty());

    let child = world
        .creatures
        .iter()
        .find(|c| c.id != mother)
        .expect("debe existir la cría");
    assert_eq!((child.x, child.y), (1, 2), "cría en la casilla objetivo");
    assert_eq!(child.health, 5, "vida de la cría = health");
    assert_eq!(child.age, 0, "edad 0 al nacer");
    assert_eq!(child.sleep_turns, 0, "sin turnos de sueño");
    assert_eq!(child.pc, 0, "puntero de ejecución al inicio (start:)");
    assert!(
        child.variables.is_empty(),
        "la cría nace sin variables, got {:?}",
        child.variables
    );
    assert_eq!(child.species.name, "Walker", "misma especie");
    assert_eq!(
        child.species.faction,
        world
            .creatures
            .iter()
            .find(|c| c.id == mother)
            .unwrap()
            .species
            .faction,
        "misma facción"
    );
    assert_eq!(
        child.species.program.is_some(),
        world
            .creatures
            .iter()
            .find(|c| c.id == mother)
            .unwrap()
            .species
            .program
            .is_some(),
        "misma presencia de programa"
    );
    assert!(
        matches!(world.content[2][1], CellContent::Creature(cid) if cid == child.id),
        "la casilla objetivo (1,2) contiene a la cría"
    );
    // La madre sigue en su casilla.
    assert_eq!(world.creature_position(mother).unwrap(), Pos { x: 1, y: 1 });
}

#[test]
fn test_reproduce_mother_loses_health() {
    // AC2: la madre pierde `health` de vida al reproducirse.
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
    assert_eq!(creature_health(&world, mother), 5, "madre pierde health");
}

#[test]
fn test_reproduce_offspring_no_variables() {
    // AC3: la cría no hereda variables de la madre.
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
        .find(|c| c.id != mother)
        .expect("debe existir la cría");
    assert!(
        child.variables.is_empty(),
        "sin variables heredadas, got {:?}",
        child.variables
    );
    assert_eq!(
        world.variable(mother, "tesoro"),
        Some(Value::Int(7)),
        "la madre conserva sus variables"
    );
}

#[test]
fn test_reproduce_abort_occupied_cell() {
    // AC5: casilla con criatura -> aborto natural, no cría, madre no pierde vida.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 10);
    let _victim = place_creature_with_health(&mut world, 1, 2, 10); // ocupa el objetivo

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(1), Value::Int(5)],
        )
        .expect("reproduce applies: aborto no es error");
    assert_eq!(r.sleep_turns, 0);
    assert!(r.events.is_empty());
    assert_eq!(
        world.creatures.len(),
        2,
        "solo la madre y la víctima, sin cría"
    );
    assert_eq!(
        creature_health(&world, mother),
        10,
        "madre no pierde vida al abortar"
    );
}

#[test]
fn test_reproduce_abort_occupied_by_object() {
    // AC5: casilla con objeto -> aborto natural, no cría.
    let (mut world, _, _) = rock_world(); // roca en (1,1)
    let mother = place_creature_with_health(&mut world, 0, 1, 10);

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(1), Value::Int(0), Value::Int(5)],
        )
        .expect("reproduce applies: aborto no es error");
    assert!(r.events.is_empty());
    assert!(
        !world.creatures.iter().any(|c| c.id != mother),
        "no debe haber cría"
    );
    assert!(matches!(world.content[1][1], CellContent::Object(0)));
    assert_eq!(creature_health(&world, mother), 10);
}

#[test]
fn test_reproduce_abort_out_of_map() {
    // AC6: destino fuera del mapa -> aborto natural, no cría.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 0, 0, 10);

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(-1), Value::Int(0), Value::Int(5)],
        )
        .expect("reproduce applies: fuera del mapa no es error");
    assert!(r.events.is_empty());
    assert!(
        world.creatures.iter().all(|c| c.id == mother),
        "no debe haber cría"
    );
    assert_eq!(creature_health(&world, mother), 10);
}

#[test]
fn test_reproduce_abort_insufficient_health() {
    // AC7: madre vida <= health -> aborto natural, no cría.
    let (mut world, _, _) = world_with_grass();
    let mother = place_creature_with_health(&mut world, 1, 1, 5);

    let reproduce = Reproduce;
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(1), Value::Int(5)],
        )
        .expect("reproduce applies: aborto no es error");
    assert!(r.events.is_empty());
    assert!(
        !world.creatures.iter().any(|c| c.id != mother),
        "no debe haber cría"
    );
    assert_eq!(creature_health(&world, mother), 5);

    // vida exactamente igual: no se cumple estricto `>`.
    let r = reproduce
        .apply(
            &mut world,
            mother,
            &[Value::Int(0), Value::Int(-1), Value::Int(5)],
        )
        .expect("reproduce applies");
    assert!(r.events.is_empty());
    assert!(
        !world.creatures.iter().any(|c| c.id != mother),
        "no debe haber cría"
    );
}

#[test]
fn test_reproduce_argument_errors_name_the_argument() {
    // dx / dy fuera de {-1,0,1} y argumentos de texto nombran el argumento.
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
