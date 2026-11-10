//! Integration tests for perceptions and variables (F0-05).
//! Uses the real world, the real parser (F0-03), and the real evaluation
//! context — no fake contexts.

use instinct_core::actions::Catalog;
use instinct_core::expr::parse;
use instinct_core::perception::{CreatureView, Perception, PerceptionCatalog};
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::{CreatureContext, CreatureId, World};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn empty_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }];
    let objects: Vec<Object> = vec![];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

fn place_walker(world: &mut World) -> CreatureId {
    world
        .place_creature(Species::new("Walker"), 4, 2)
        .expect("place walker")
}

fn creature_mut(world: &mut World, id: CreatureId) -> &mut instinct_core::world::Creature {
    world
        .creatures
        .iter_mut()
        .find(|c| c.id == id)
        .expect("creature exists")
}

fn set_vision(world: &mut World, id: CreatureId, vision: i64) {
    for c in world.creatures.iter_mut() {
        if c.id == id {
            c.species.vision = vision;
        }
    }
}

fn eval(world: &World, id: CreatureId, text: &str) -> Result<Value, String> {
    let expr = parse(text, 1, "test.ins", Catalog::default().functions()).expect("parse");
    let ctx = CreatureContext::new(world, id, Rc::new(RefCell::new(HashMap::new())));
    expr.eval(&ctx)
}

fn unwrap_int(v: Value) -> i64 {
    match v {
        Value::Int(n) => n,
        Value::Text(_) => panic!("expected an integer, got {v:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC1: the seven perceptions report the creature's state
// ---------------------------------------------------------------------------

#[test]
fn test_perceptions_report_creature_state() {
    let mut world = empty_world();
    let id = place_walker(&mut world);
    let c = creature_mut(&mut world, id);
    c.health = 10;
    c.age = 3;
    c.lifespan = 50;
    world.ticks = 7;

    let cases: &[(&str, i64)] = &[
        ("health", 10),
        ("age", 3),
        ("lifespan", 50),
        ("x", 4),
        ("y", 2),
        ("tick", 7),
    ];
    for (name, expected) in cases {
        assert_eq!(
            unwrap_int(eval(&world, id, name).unwrap()),
            *expected,
            "perception {name}"
        );
    }

    // random is an integer between 0 and 99
    let v = eval(&world, id, "random").unwrap();
    assert!(
        matches!(v, Value::Int(n) if (0..=99).contains(&n)),
        "random in 0..=99, got {v:?}"
    );
}

// ---------------------------------------------------------------------------
// AC2: expressions read perceptions
// ---------------------------------------------------------------------------

#[test]
fn test_expression_reads_perceptions() {
    let mut world = empty_world();
    let id = place_walker(&mut world);
    let c = creature_mut(&mut world, id);
    c.health = 10;
    c.x = 4;

    // health * 2 + x = 10 * 2 + 4 = 24
    assert_eq!(eval(&world, id, "health * 2 + x").unwrap(), Value::Int(24));

    // tick in a freshly created world is 0
    let mut fresh = empty_world();
    let id2 = place_walker(&mut fresh);
    assert_eq!(eval(&fresh, id2, "tick").unwrap(), Value::Int(0));
}

// ---------------------------------------------------------------------------
// AC3: random advances the generator once per read; deterministic per seed
// ---------------------------------------------------------------------------

#[test]
fn test_random_consumes_one_draw_per_read() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    world.set_rng_seed(42);
    let d1 = unwrap_int(eval(&world, id, "random").unwrap());
    let d2 = unwrap_int(eval(&world, id, "random").unwrap());
    let d3 = unwrap_int(eval(&world, id, "random").unwrap());

    // Same seed reproduces the same sequence.
    world.set_rng_seed(42);
    let e1 = unwrap_int(eval(&world, id, "random").unwrap());
    let e2 = unwrap_int(eval(&world, id, "random").unwrap());
    assert_eq!(e1, d1);
    assert_eq!(e2, d2);

    // Two reads in one expression consume two draws: random + random == d1 + d2,
    // and the following read is d3 (not d2).
    world.set_rng_seed(42);
    let sum = unwrap_int(eval(&world, id, "random + random").unwrap());
    let next = unwrap_int(eval(&world, id, "random").unwrap());
    assert_eq!(sum, d1 + d2);
    assert_eq!(next, d3);
}

// ---------------------------------------------------------------------------
// AC4: assign, read, retype; unassigned variable errors naming it
// ---------------------------------------------------------------------------

#[test]
fn test_variables_assign_read_and_retype() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    world
        .set_variable(id, "score", Value::Int(41))
        .expect("assign score");
    assert_eq!(eval(&world, id, "score").unwrap(), Value::Int(41));

    // A variable can change from integer to text on reassignment.
    world
        .set_variable(id, "score", Value::Text("listo".to_string()))
        .expect("reassign score");
    assert_eq!(
        eval(&world, id, "score").unwrap(),
        Value::Text("listo".to_string())
    );
}

#[test]
fn test_unassigned_variable_error() {
    let mut world = empty_world();
    let id = place_walker(&mut world);
    let err = eval(&world, id, "missing_var").unwrap_err();
    assert!(
        err.contains("missing_var"),
        "error names the variable: {err}"
    );
}

// ---------------------------------------------------------------------------
// AC5: variables are per creature
// ---------------------------------------------------------------------------

#[test]
fn test_variables_are_per_creature() {
    let mut world = empty_world();
    let id1 = place_walker(&mut world);
    let id2 = world
        .place_creature(Species::new("Walker2"), 0, 0)
        .expect("place second walker");

    world
        .set_variable(id1, "energia", Value::Int(1))
        .expect("assign creature 1");
    world
        .set_variable(id2, "energia", Value::Int(2))
        .expect("assign creature 2");

    assert_eq!(eval(&world, id1, "energia").unwrap(), Value::Int(1));
    assert_eq!(eval(&world, id2, "energia").unwrap(), Value::Int(2));
}

// ---------------------------------------------------------------------------
// AC6: cannot assign to a perception or action name
// ---------------------------------------------------------------------------

#[test]
fn test_cannot_assign_perception() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    let err = world
        .set_variable(id, "health", Value::Int(99))
        .unwrap_err();
    assert!(
        err.message.contains("health"),
        "error names health: {err:?}"
    );

    let err = world.set_variable(id, "wait", Value::Int(1)).unwrap_err();
    assert!(err.message.contains("wait"), "error names wait: {err:?}");
}

// ---------------------------------------------------------------------------
// AC7: the catalog accepts an extra perception defined in the test
// ---------------------------------------------------------------------------

#[test]
fn test_catalog_accepts_extra_perception() {
    #[derive(Debug, Clone, Copy)]
    struct Bonus;

    impl Perception for Bonus {
        fn name(&self) -> &str {
            "bonus"
        }
        fn read(&self, _view: &dyn CreatureView) -> Value {
            Value::Int(7)
        }
        fn clone_box(&self) -> Box<dyn Perception> {
            Box::new(*self)
        }
    }

    #[derive(Debug)]
    struct EmptyView;

    impl CreatureView for EmptyView {
        fn health(&self) -> i64 {
            0
        }
        fn age(&self) -> i64 {
            0
        }
        fn lifespan(&self) -> i64 {
            0
        }
        fn x(&self) -> i64 {
            0
        }
        fn y(&self) -> i64 {
            0
        }
        fn tick(&self) -> u64 {
            0
        }
        fn random(&self) -> i64 {
            0
        }
        fn vision(&self) -> i64 {
            0
        }
        fn species_name(&self) -> &str {
            ""
        }
        fn faction_name(&self) -> &str {
            ""
        }
        fn terrain_here(&self) -> &str {
            ""
        }
        fn reserve_here(&self) -> i64 {
            0
        }
        fn width(&self) -> i64 {
            0
        }
        fn height(&self) -> i64 {
            0
        }
        fn total_creatures(&self) -> i64 {
            0
        }
        fn total_allies(&self) -> i64 {
            0
        }
        fn total_enemies(&self) -> i64 {
            0
        }
        fn nearby(&self) -> instinct_core::perception::Nearby {
            instinct_core::perception::Nearby::empty()
        }

        fn see(&self, _x: i64, _y: i64) -> i64 {
            0
        }

        fn name(&self, _x: i64, _y: i64) -> &str {
            ""
        }
    }

    let mut cat = Catalog::default();
    assert!(
        cat.resolve_perception("bonus").is_none(),
        "bonus not registered yet"
    );
    // The seven built-in perceptions are still present.
    for name in ["health", "age", "lifespan", "x", "y", "tick", "random"] {
        assert!(
            cat.resolve_perception(name).is_some(),
            "{name} still registered"
        );
    }

    assert!(
        cat.register_perception(Box::new(Bonus)).is_ok(),
        "register bonus"
    );
    let view = EmptyView;
    let p = cat.resolve_perception("bonus").expect("bonus resolves");
    assert_eq!(p.read(&view), Value::Int(7));
}

// ---------------------------------------------------------------------------
// AC2: species, faction y vision de la propia criatura
// ---------------------------------------------------------------------------

#[test]
fn test_self_perceptions_report_species_data() {
    let mut world = empty_world();
    let id = place_walker(&mut world); // (4, 2) sobre Grass
    let c = creature_mut(&mut world, id);
    c.species.name = "Hobbit".to_string();
    c.species.faction = "Comarca".to_string();
    c.species.vision = 2;

    assert_eq!(
        eval(&world, id, "species").unwrap(),
        Value::Text("Hobbit".to_string())
    );
    assert_eq!(
        eval(&world, id, "faction").unwrap(),
        Value::Text("Comarca".to_string())
    );
    assert_eq!(unwrap_int(eval(&world, id, "vision").unwrap()), 2);
}

// ---------------------------------------------------------------------------
// AC3: terrain_here y reserve_here leen la casilla bajo la criatura
// ---------------------------------------------------------------------------

#[test]
fn test_terrain_here_tracks_reserve() {
    let mut world = empty_world(); // 5x5 Grass, reserva 100
    let id = place_walker(&mut world); // (4, 2)

    assert_eq!(
        eval(&world, id, "terrain_here").unwrap(),
        Value::Text("Grass".to_string())
    );
    assert_eq!(unwrap_int(eval(&world, id, "reserve_here").unwrap()), 100);

    // Retira la criatura para dañar el terreno bajo ella sin lastimarla:
    // damage_cell_content sobre una casilla con criatura no toca el terreno.
    world.remove_creature(id).expect("retirar criatura");
    world
        .damage_cell_content(4, 2, 30)
        .expect("daña terreno de (4,2)");
    assert_eq!(world.terrains[0].reserve, 70);

    // Vuelve a colocar en la misma casilla (nuevo id) y lee la reserva dañada.
    let id2 = world
        .place_creature(Species::new("Walker"), 4, 2)
        .expect("colocar de nuevo");
    assert_eq!(unwrap_int(eval(&world, id2, "reserve_here").unwrap()), 70);

    // Fin de tick: regenera +2 sobre la reserva dañada: 72 (topada en 100).
    world.end_of_tick_terrain_and_objects();
    assert_eq!(unwrap_int(eval(&world, id2, "reserve_here").unwrap()), 72);
}

// ---------------------------------------------------------------------------
// AC4: width, height y totales sobre un mundo de varias facciones
// ---------------------------------------------------------------------------

#[test]
fn test_totals_count_world_excluding_self() {
    let mut world = empty_world(); // 5x5

    let mut faction_a = Species::new("A");
    faction_a.faction = "A".to_string();
    let id_self = world
        .place_creature(faction_a.clone(), 0, 0)
        .expect("colocar self");

    let mut faction_b = Species::new("B");
    faction_b.faction = "B".to_string();
    let _id_ally = world
        .place_creature(faction_a.clone(), 1, 0)
        .expect("colocar aliado");
    let _id_enemy1 = world
        .place_creature(faction_b.clone(), 2, 0)
        .expect("colocar enemigo 1");
    let _id_enemy2 = world
        .place_creature(faction_b, 3, 0)
        .expect("colocar enemigo 2");

    assert_eq!(unwrap_int(eval(&world, id_self, "width").unwrap()), 5);
    assert_eq!(unwrap_int(eval(&world, id_self, "height").unwrap()), 3);
    // Cuenta a las otras 3 criaturas, no a sí misma.
    assert_eq!(
        unwrap_int(eval(&world, id_self, "total_creatures").unwrap()),
        3
    );
    // Un aliado de la facción A (sin contarse a sí misma).
    assert_eq!(
        unwrap_int(eval(&world, id_self, "total_allies").unwrap()),
        1
    );
    // Dos enemigos de la facción B.
    assert_eq!(
        unwrap_int(eval(&world, id_self, "total_enemies").unwrap()),
        2
    );
}

// ---------------------------------------------------------------------------
// AC5: los diez nombres nuevos resuelven en ambos catálogos por defecto
// ---------------------------------------------------------------------------

#[test]
fn test_default_catalogs_resolve_self_and_world_perceptions() {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    for name in [
        "vision",
        "species",
        "faction",
        "terrain_here",
        "reserve_here",
        "width",
        "height",
        "total_creatures",
        "total_allies",
        "total_enemies",
    ] {
        assert!(
            cat.resolve_perception(name).is_some(),
            "Catalog::default() debe registrar {name}"
        );
        assert!(
            pc.resolve(name).is_some(),
            "PerceptionCatalog::default() debe registrar {name}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC1: near counts exclude own cell
// ---------------------------------------------------------------------------

#[test]
fn test_near_counts_exclude_own_cell() {
    let mut world = empty_world(); // 5x5 grass
    let id = place_walker(&mut world); // (4, 2)
    set_vision(&mut world, id, 2);

    // Two allies at (3, 2) and (2, 2) — both distance <= 2
    let _id_ally1 = world
        .place_creature(Species::new("Ally1"), 3, 2)
        .expect("place ally 1");
    let _id_ally2 = world
        .place_creature(Species::new("Ally2"), 2, 2)
        .expect("place ally 2");

    // One enemy at (1, 2) — distance 3 from (4, 2), out of vision 2
    // Let me use distance 2 instead: (2, 2) is distance 2, but that's where ally2 is.
    // Place enemy at (0, 2): distance = max(4, 0) = 4 > 2, out of range.
    // Place enemy at (3, 0): distance = max(1, 2) = 2 <= 2, in range.
    let mut sp_e = Species::new("Enemy");
    sp_e.faction = "B".to_string();
    let _id_enemy = world
        .place_creature(sp_e, 3, 0)
        .expect("place enemy at distance 2");

    // One object: empty_world has no objects on grid. Place one manually.
    // content[row][col]: row=1, col=3 => position (3, 1). Distance from (4,2): max(1, 1) = 1 <= 2.
    world.content[1][3] = instinct_core::world::CellContent::Object(0);

    assert_eq!(
        unwrap_int(eval(&world, id, "allies_near").unwrap()),
        2,
        "two allies within vision, self excluded"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemies_near").unwrap()),
        1,
        "one enemy within vision"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "objects_near").unwrap()),
        1,
        "one object within vision"
    );
}

// ---------------------------------------------------------------------------
// AC3: nearest target reports distance and direction
// ---------------------------------------------------------------------------

#[test]
fn test_nearest_target_reports_direction() {
    let mut world = empty_world(); // 5x5 grass
    let id = place_walker(&mut world); // (4, 2)
    set_vision(&mut world, id, 3);

    // Enemy at (1, 2): distance = max(3, 0) = 3 = vision, IN
    // dx = 1-4 = -3 -> signum -1, dy = 0
    let mut sp_e = Species::new("Enemy");
    sp_e.faction = "B".to_string();
    let _id_enemy = world
        .place_creature(sp_e, 1, 2)
        .expect("place enemy at distance 3");

    // Ally at (3, 2): distance = max(1, 0) = 1
    // dx = -1, dy = 0
    let _id_ally = world
        .place_creature(Species::new("Ally"), 3, 2)
        .expect("place ally at distance 1");

    // Object at (4, 0): distance = max(0, 2) = 2
    // content[row][col]: row=0, col=4 => position (4, 0). Distance from (4,2): max(0, 2) = 2.
    // dx = 0, dy = -2 -> signum -1
    world.content[0][4] = instinct_core::world::CellContent::Object(0);

    // Enemy: dist 3, dx -1, dy 0
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dist").unwrap()),
        3,
        "enemy distance"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dx").unwrap()),
        -1,
        "enemy dx"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dy").unwrap()),
        0,
        "enemy dy"
    );

    // Ally: dist 1, dx -1, dy 0
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dist").unwrap()),
        1,
        "ally distance"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dx").unwrap()),
        -1,
        "ally dx"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dy").unwrap()),
        0,
        "ally dy"
    );

    // Object: dist 2, dx 0, dy -1
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dist").unwrap()),
        2,
        "object distance"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dx").unwrap()),
        0,
        "object dx"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dy").unwrap()),
        -1,
        "object dy"
    );
}

// ---------------------------------------------------------------------------
// No visible target: dist -1, directions 0
// ---------------------------------------------------------------------------

#[test]
fn test_nearest_target_none_visible() {
    let mut world = empty_world(); // 5x5 grass, no other creatures
    let id = place_walker(&mut world); // (4, 2)
    set_vision(&mut world, id, 1);

    // No allies, enemies, or objects within vision 1
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dist").unwrap()),
        -1,
        "no enemy: dist -1"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dx").unwrap()),
        0,
        "no enemy: dx 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "enemy_dy").unwrap()),
        0,
        "no enemy: dy 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dist").unwrap()),
        -1,
        "no ally: dist -1"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dx").unwrap()),
        0,
        "no ally: dx 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "ally_dy").unwrap()),
        0,
        "no ally: dy 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dist").unwrap()),
        -1,
        "no object: dist -1"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dx").unwrap()),
        0,
        "no object: dx 0"
    );
    assert_eq!(
        unwrap_int(eval(&world, id, "object_dy").unwrap()),
        0,
        "no object: dy 0"
    );
}

// ---------------------------------------------------------------------------
// AC7: all 31 perceptions resolve in both default catalogs
// ---------------------------------------------------------------------------

#[test]
fn test_default_catalogs_resolve_all_31_perceptions() {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let all_31 = [
        // 7 original (F0)
        "health",
        "age",
        "lifespan",
        "x",
        "y",
        "tick",
        "random",
        // 10 self/world (F3-01)
        "vision",
        "species",
        "faction",
        "terrain_here",
        "reserve_here",
        "width",
        "height",
        "total_creatures",
        "total_allies",
        "total_enemies",
        // 14 nearby (F3-02)
        "allies_near",
        "enemies_near",
        "objects_near",
        "enemy_dist",
        "enemy_dx",
        "enemy_dy",
        "ally_dist",
        "ally_dx",
        "ally_dy",
        "object_dist",
        "object_dx",
        "object_dy",
        "roars_near",
        "last_roar",
    ];
    for name in all_31 {
        assert!(
            cat.resolve_perception(name).is_some(),
            "Catalog::default() debe registrar {name}"
        );
        assert!(
            pc.resolve(name).is_some(),
            "PerceptionCatalog::default() debe registrar {name}"
        );
    }
}
