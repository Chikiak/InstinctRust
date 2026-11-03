//! Scenario tests for the Simulation (sim.rs).
//! Covers AC5 (death), AC6 (terrain/objects end-of-tick), AC8 (determinism).

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::Event;
use instinct_core::perception::PerceptionCatalog;
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world::{CellContent, World};
use std::rc::Rc;

fn grass_terrain() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

fn rock_object() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

fn empty_map() -> Map {
    Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    }
}

/// Compiles an `.ins` body and returns a Species with the compiled program.
fn species_with_program(name: &str, health: i64, lifespan: i64, body: &str) -> Species {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text = format!(
        "creature {name}\nfaction any\nhealth {health}\nvision 3\nlifespan {lifespan}\nstart:\n{body}"
    );
    let result = compile::compile(&text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("header");
    Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    )
}

fn species_no_program(name: &str, health: i64, lifespan: i64) -> Species {
    Species {
        name: name.to_string(),
        faction: "any".to_string(),
        health,
        vision: 3,
        lifespan,
        program: None,
    }
}

fn build_world(map: &Map, terrains: &[Terrain], objects: &[Object]) -> World {
    let world = World::from_map(map, terrains, objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

// ---------------------------------------------------------------------------
// AC5: death by health and by age.
// ---------------------------------------------------------------------------

#[test]
fn test_death_by_health_and_by_age() {
    // Creature A: health 10, lifespan 100 -> dies by salud at end of tick 10.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_a = species_no_program("A", 10, 100);
    let id_a = world.place_creature(sp_a, 0, 0).expect("place A");

    let mut sim = Simulation::new(world, vec![grass_terrain()], vec![]);

    // 9 ticks: alive, health goes 10 -> 1.
    for _ in 0..9 {
        let events = sim.step_tick();
        assert!(
            events.iter().all(|e| !matches!(e, Event::Died { .. })),
            "no death expected before tick 10, got: {events:?}"
        );
    }
    let creature = sim
        .world
        .creatures
        .iter()
        .find(|c| c.id == id_a)
        .expect("creature alive after 9 ticks");
    assert_eq!(creature.health, 1);
    assert_eq!(creature.age, 9);

    // Tick 10: health 1 -> 0, dies by vida.
    let events = sim.step_tick();
    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(died.len(), 1);
    match &died[0] {
        Event::Died {
            tick,
            species,
            cause,
            ..
        } => {
            assert_eq!(*tick, 9); // tick counter is 9 before incrementing to 10
            assert_eq!(species, "A");
            assert_eq!(cause, "vida");
        }
        _ => panic!("expected Died event"),
    }
    assert!(sim.world.creatures.iter().all(|c| c.id != id_a));

    // Creature B: health 100, lifespan 5 -> dies by edad at end of tick 5.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_b = species_no_program("B", 100, 5);
    let id_b = world.place_creature(sp_b, 0, 0).expect("place B");
    let mut sim = Simulation::new(world, vec![grass_terrain()], vec![]);

    for _ in 0..4 {
        let events = sim.step_tick();
        assert!(
            events.iter().all(|e| !matches!(e, Event::Died { .. })),
            "no death expected before tick 5"
        );
    }
    let creature = sim
        .world
        .creatures
        .iter()
        .find(|c| c.id == id_b)
        .expect("creature alive after 4 ticks");
    assert_eq!(creature.age, 4);

    let events = sim.step_tick();
    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(died.len(), 1);
    match &died[0] {
        Event::Died {
            tick,
            species,
            cause,
            ..
        } => {
            assert_eq!(*tick, 4);
            assert_eq!(species, "B");
            assert_eq!(cause, "edad");
        }
        _ => panic!("expected Died event"),
    }
    assert!(sim.world.creatures.iter().all(|c| c.id != id_b));
}

// ---------------------------------------------------------------------------
// AC6: terrain regenerates (capped), objects with reserve 0 vanish.
// ---------------------------------------------------------------------------

#[test]
fn test_terrain_regenerates_and_empty_objects_vanish() {
    // Map with grass everywhere and a rock at (2,2).
    let mut map = empty_map();
    map.rows[2][2] = '#';
    let rock = rock_object();
    let world = build_world(&map, &[grass_terrain()], std::slice::from_ref(&rock));
    assert!(matches!(world.content[2][2], CellContent::Object(0)));

    let mut sim = Simulation::new(world, vec![grass_terrain()], vec![rock]);
    // Set object reserve to 0 so it vanishes.
    sim.objects[0].reserve = 0;
    // Drain terrain reserve to 0.
    sim.terrains[0].reserve = 0;

    let _events = sim.step_tick();

    // Terrain at (0,0) had reserve 0, regen 2 -> now 2 (capped at 100).
    assert_eq!(sim.terrains[0].reserve, 2);
    // Object at (2,2) had reserve 0 -> vanished.
    assert!(matches!(sim.world.content[2][2], CellContent::Empty));
}

// ---------------------------------------------------------------------------
// AC8: determinism and seed-dependent turn order.
// ---------------------------------------------------------------------------

fn walker_species() -> Species {
    species_with_program(
        "Walker",
        100,
        1000,
        "move(random % 3 - 1, random % 3 - 1, 1)\ngoto start\n",
    )
}

fn two_walker_world() -> (World, Vec<Terrain>, Vec<Object>) {
    // 5x3 map, all grass. Walker A at (0,0), Walker B at (1,0).
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    };
    let mut world = build_world(&map, &[grass_terrain()], &[]);
    let sp = walker_species();
    let id_a = world.place_creature(sp.clone(), 0, 0).expect("place A");
    let id_b = world.place_creature(sp, 1, 0).expect("place B");
    assert_ne!(id_a, id_b);
    (world, vec![grass_terrain()], vec![])
}

fn run_ticks(sim: &mut Simulation, n: u64) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..n {
        all.extend(sim.step_tick());
    }
    all
}

#[test]
fn test_same_seed_same_history() {
    let (world, terrains, objects) = two_walker_world();
    let mut sim1 = Simulation::new(world.clone(), terrains.clone(), objects.clone());
    sim1.set_rng_seed(42);
    let events1 = run_ticks(&mut sim1, 50);
    let map1 = sim1.world.draw(&sim1.terrains, &sim1.objects);

    let mut sim2 = Simulation::new(world, terrains, objects);
    sim2.set_rng_seed(42);
    let events2 = run_ticks(&mut sim2, 50);
    let map2 = sim2.world.draw(&sim2.terrains, &sim2.objects);

    assert_eq!(events1, events2, "same seed must produce same events");
    assert_eq!(map1, map2, "same seed must produce same map");
}

#[test]
fn test_seed_changes_turn_order() {
    // Two walkers: A at (0,0), B at (1,0). Both try to move right.
    // A wants (1,0) [occupied by B], B wants (2,0) [free].
    // If B moves first, (1,0) frees up and A can move there next turn.
    // If A moves first, A is blocked, B moves to (2,0).

    let (world, terrains, objects) = two_walker_world();
    let mut sim1 = Simulation::new(world.clone(), terrains.clone(), objects.clone());
    sim1.set_rng_seed(42);
    let events1 = run_ticks(&mut sim1, 50);
    let map1 = sim1.world.draw(&sim1.terrains, &sim1.objects);

    let mut sim2 = Simulation::new(world, terrains, objects);
    sim2.set_rng_seed(99);
    let events2 = run_ticks(&mut sim2, 50);
    let map2 = sim2.world.draw(&sim2.terrains, &sim2.objects);

    // The maps should differ after 50 ticks with different seeds.
    assert!(
        map1 != map2 || events1 != events2,
        "different seeds should change turn order and thus the outcome:\nmap1:\n{map1}\nmap2:\n{map2}"
    );
}
