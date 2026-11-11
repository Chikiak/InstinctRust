//! Scenario tests for the four annex examples (F3-04).
//! Real world, real catalog, fixed seed; one scenario per creature asserting
//! events and positions, not text output.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::{Event, Pos};
use instinct_core::perception::PerceptionCatalog;
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::rng::Rng;
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world::World;
use std::rc::Rc;

fn examples_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
}

/// Compiles an annex `.ins` and returns la `Species` con su programa,
/// envuelta en `Rc` para compartirla por el catálogo del mundo (Flyweight).
fn load_annex_creature(name: &str) -> Rc<Species> {
    let path = format!("{}/creatures/{}.ins", examples_dir(), name);
    let content =
        std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{name}.ins must exist"));
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let result = compile::compile(&content, &cat, &pc, &path);
    assert!(
        result.diagnostics.is_empty(),
        "{name}.ins should compile without diagnostics, got: {:?}",
        result.diagnostics
    );
    let header = result
        .header
        .unwrap_or_else(|| panic!("{name}.ins should have a header"));
    Rc::new(Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    ))
}

fn grass_terrain() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

fn empty_map() -> Map {
    Map {
        name: "test".to_string(),
        rows: vec![
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
        ],
    }
}

fn build_world(map: &Map, terrains: &[Terrain], objects: &[Object]) -> World {
    let world = World::from_map(map, terrains, objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

fn species_no_program(name: &str, health: i64, lifespan: i64) -> Rc<Species> {
    Rc::new(Species {
        name: name.to_string(),
        faction: "any".to_string(),
        health,
        vision: 3,
        lifespan,
        program: None,
    })
}

/// Runs `n` ticks and returns all emitted events.
fn run_ticks(sim: &mut Simulation, n: u64) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..n {
        all.extend(sim.step_tick());
    }
    all
}

fn has_runtime_error(events: &[Event]) -> bool {
    events
        .iter()
        .any(|e| matches!(e, Event::RuntimeError { .. }))
}

// ---------------------------------------------------------------------------
// AC1: the four annex creatures compile without diagnostics
// ---------------------------------------------------------------------------

#[test]
fn test_annex_creatures_compile() {
    for name in ["hobbit", "ent", "dwarf", "uruk"] {
        let _ = load_annex_creature(name);
    }
}

// ---------------------------------------------------------------------------
// AC2: Hobbit flees and speaks
// ---------------------------------------------------------------------------

#[test]
fn test_hobbit_flees_and_speaks() {
    // Hobbit at (2,2), enemy (other faction) at (3,2): enemy_dist == 1 >= 0.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let hobbit = load_annex_creature("hobbit");
    let hobbit_id = world.place_creature(&hobbit, 2, 2).expect("place hobbit");
    let enemy = species_no_program("Enemy", 20, 100);
    let _enemy_id = world.place_creature(&enemy, 3, 2).expect("place enemy");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Said the flee line.
    assert!(
        events.iter().any(
            |e| matches!(e, Event::Said { text, .. } if text == "they are taking us to Isengard")
        ),
        "hobbit should say the flee line, got: {events:?}"
    );

    // No runtime error.
    assert!(
        !has_runtime_error(&events),
        "no runtime error, got: {events:?}"
    );

    // The hobbit moved away from the enemy (was at (2,2), enemy at (3,2),
    // distance 1; now at (0,2), distance 3).
    let hobbit = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == hobbit_id)
        .expect("hobbit alive");
    assert_eq!(
        (hobbit.x, hobbit.y),
        (0, 2),
        "hobbit should have moved away"
    );
}

// ---------------------------------------------------------------------------
// AC3: Ent feeds on its terrain and stays put
// ---------------------------------------------------------------------------

#[test]
fn test_ent_feeds_and_stays() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let ent = load_annex_creature("ent");
    let ent_id = world.place_creature(&ent, 2, 2).expect("place ent");
    // health < 150 so the Ent goes to `roots` and eats.
    world
        .set_creature_health(ent_id, 140)
        .expect("set ent health");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Damaged event of 3 on its own cell.
    assert!(
        events.iter().any(|e| {
            matches!(
                e,
                Event::Damaged {
                    amount, target, ..
                } if *amount == 3 && *target == Pos { x: 2, y: 2 }
            )
        }),
        "ent should damage its own terrain by 3, got: {events:?}"
    );

    // No runtime error.
    assert!(
        !has_runtime_error(&events),
        "no runtime error, got: {events:?}"
    );

    // The Ent did not leave its cell.
    let ent = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == ent_id)
        .expect("ent alive");
    assert_eq!((ent.x, ent.y), (2, 2), "ent should stay on its cell");
}

// ---------------------------------------------------------------------------
// AC4: Dwarf mines an adjacent rock
// ---------------------------------------------------------------------------

#[test]
fn test_dwarf_mines_adjacent_rock() {
    // Dwarf at (0,0), a rock with reserve 20 at (1,0).
    let mut map = empty_map();
    map.rows[0][1] = '#';
    let rock = Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 20,
        reserve: 20,
    };
    let mut world = build_world(&map, &[grass_terrain()], std::slice::from_ref(&rock));
    assert!(matches!(
        world.content[0][1],
        instinct_core::world::CellContent::Object(0)
    ));

    let dwarf = load_annex_creature("dwarf");
    // health < 100 so `object_dist == 1 and health < 100` leads to `mine`.
    let dwarf_id = world.place_creature(&dwarf, 0, 0).expect("place dwarf");
    world
        .set_creature_health(dwarf_id, 90)
        .expect("set dwarf health");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Damaged event of 20 on the rock's cell.
    assert!(
        events.iter().any(|e| {
            matches!(
                e,
                Event::Damaged {
                    amount, target, ..
                } if *amount == 20 && *target == Pos { x: 1, y: 0 }
            )
        }),
        "dwarf should damage the rock by 20, got: {events:?}"
    );

    // No runtime error.
    assert!(
        !has_runtime_error(&events),
        "no runtime error, got: {events:?}"
    );

    // The rock is exhausted: its cell is free.
    assert!(
        matches!(
            sim.world.content[0][1],
            instinct_core::world::CellContent::Empty
        ),
        "rock cell should be free, got {:?}",
        sim.world.content[0][1]
    );
}

// ---------------------------------------------------------------------------
// AC5: Uruk hunts and breeds
// ---------------------------------------------------------------------------

#[test]
fn test_uruk_hunts_and_breeds() {
    // --- Hunt: Uruk consumes an adjacent enemy -----------------------------
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let uruk = load_annex_creature("uruk");
    let _uruk_id = world.place_creature(&uruk, 0, 0).expect("place uruk");
    let enemy = species_no_program("Enemy", 10, 100);
    let _enemy_id = world.place_creature(&enemy, 1, 0).expect("place enemy");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Consumes the adjacent enemy (damage 15) and the enemy dies.
    assert!(
        events.iter().any(|e| {
            matches!(
                e,
                Event::Damaged {
                    amount, target, ..
                } if *amount == 15 && *target == Pos { x: 1, y: 0 }
            )
        }),
        "uruk should damage the adjacent enemy by 15, got: {events:?}"
    );
    assert!(
        events.iter().any(|e| matches!(e, Event::Died { .. })),
        "enemy should die, got: {events:?}"
    );
    assert!(
        !has_runtime_error(&events),
        "no runtime error while hunting, got: {events:?}"
    );
    assert!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .all(|c| c.species.name != "Enemy"),
        "enemy removed from the map"
    );

    // --- Breed: Uruk alone breeds when conditions hold ---------------------
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let uruk = load_annex_creature("uruk");
    let uruk_id = world.place_creature(&uruk, 2, 2).expect("place uruk");
    world
        .set_creature_health(uruk_id, 80)
        .expect("set uruk health");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);

    // Tick 0: breeds below.
    let events = sim.step_tick();
    assert!(
        !has_runtime_error(&events),
        "no runtime error while breeding, got: {events:?}"
    );
    let child = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != uruk_id)
        .expect("a child should exist");
    assert_eq!((child.x, child.y), (2, 3), "child below the mother");
    assert_eq!(child.species.name, "Uruk");
    let mother = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == uruk_id)
        .expect("mother alive");
    assert!(mother.health < 80, "mother spent health to breed");

    // --- 30 ticks without any RuntimeError ---------------------------------
    let events = run_ticks(&mut sim, 30);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::RuntimeError { .. })),
        "no runtime error in 30 ticks, got: {events:?}"
    );
}

#[test]
fn test_annex_seed7_sequence_unchanged() {
    // AC4: No-regresión. Rng::new(7) produce la misma secuencia de
    // next_bounded y shuffle que antes del issue (valores capturados en la
    // versión base).
    let mut rng = Rng::new(7);
    assert_eq!(rng.next_bounded(99), 87);
    assert_eq!(rng.next_bounded(99), 4);
    assert_eq!(rng.next_bounded(99), 46);
    assert_eq!(rng.next_bounded(99), 3);
    assert_eq!(rng.next_bounded(99), 74);
    assert_eq!(rng.next_bounded(99), 5);
    assert_eq!(rng.next_bounded(99), 98);
    assert_eq!(rng.next_bounded(99), 82);

    // Shuffle con seed 7: índices [0..6] -> [2, 4, 1, 3, 0, 5].
    let mut order: Vec<usize> = (0..6).collect();
    rng.shuffle(&mut order);
    assert_eq!(order, vec![2, 4, 1, 3, 0, 5]);

    // Los cuatro ejemplos con --seed 7 son deterministas: dos corridas iguales.
    for name in ["hobbit", "ent", "dwarf", "uruk"] {
        let mut world1 = build_world(&empty_map(), &[grass_terrain()], &[]);
        let creature = load_annex_creature(name);
        let _id = world1
            .place_creature(&creature, 2, 2)
            .expect("place creature");
        world1.set_rng_seed(7);
        let mut sim1 = Simulation::new(world1);
        let events1 = run_ticks(&mut sim1, 10);

        let mut world2 = build_world(&empty_map(), &[grass_terrain()], &[]);
        let creature2 = load_annex_creature(name);
        let _id2 = world2
            .place_creature(&creature2, 2, 2)
            .expect("place creature");
        world2.set_rng_seed(7);
        let mut sim2 = Simulation::new(world2);
        let events2 = run_ticks(&mut sim2, 10);

        assert_eq!(
            events1, events2,
            "seed 7 determinista para {}:\n{events1:?}\n{events2:?}",
            name
        );
    }
}
