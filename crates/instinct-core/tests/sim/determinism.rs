//! Determinism scenario tests, extracted from `tests/sim_tests.rs`.
//! Keeps all test logic identical; comments translated to English.

use super::common::*;
use instinct_core::event::Event;
use instinct_core::resources::{Map, Terrain};
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world::World;
use std::cell::RefCell;
use std::rc::Rc;

fn walker_species() -> Rc<Species> {
    species_with_program(
        "Walker",
        100,
        1000,
        "move(random % 3 - 1, random % 3 - 1, 1)\ngoto start\n",
    )
}

fn two_walker_world() -> World {
    // 5x3 map, all grass. Walker A at (0,0), Walker B at (1,0).
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    };
    let mut world = build_world(&map, &[grass_terrain()], &[]);
    let sp = walker_species();
    let id_a = world.place_creature(&sp, 0, 0).expect("place A");
    let id_b = world.place_creature(&sp, 1, 0).expect("place B");
    assert_ne!(id_a, id_b);
    world
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
    let world = two_walker_world();
    let mut sim1 = Simulation::new(world.clone());
    sim1.set_rng_seed(42);
    let events1 = run_ticks(&mut sim1, 50);
    let map1 = sim1.world.draw().unwrap();

    let mut sim2 = Simulation::new(world);
    sim2.set_rng_seed(42);
    let events2 = run_ticks(&mut sim2, 50);
    let map2 = sim2.world.draw().unwrap();

    assert_eq!(events1, events2, "same seed must produce same events");
    assert_eq!(map1, map2, "same seed must produce same map");
}

#[test]
fn test_seed_changes_turn_order() {
    // Two walkers: A at (0,0), B at (1,0). Both try to move right.
    // A wants (1,0) [occupied by B], B wants (2,0) [free].
    // If B moves first, (1,0) frees up and A can move there next turn.
    // If A moves first, A is blocked, B moves to (2,0).

    let world = two_walker_world();
    let mut sim1 = Simulation::new(world.clone());
    sim1.set_rng_seed(42);
    let events1 = run_ticks(&mut sim1, 50);
    let map1 = sim1.world.draw().unwrap();

    let mut sim2 = Simulation::new(world);
    sim2.set_rng_seed(99);
    let events2 = run_ticks(&mut sim2, 50);
    let map2 = sim2.world.draw().unwrap();

    // The maps should differ after 50 ticks with different seeds.
    assert!(
        map1 != map2 || events1 != events2,
        "different seeds should change turn order and thus the outcome:\nmap1:\n{map1}\nmap2:\n{map2}"
    );
}

#[test]
fn test_near_perceptions_are_deterministic() {
    // Two creatures use enemy_dist/ally_dist to decide movement. Same seed
    // -> same history.
    fn make_world() -> World {
        let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
        let sp_a = species_with_program_and_faction(
            "A",
            "F1",
            100,
            1000,
            "say(enemy_dist)\nsay(ally_dist)\nwait(1)\ngoto start\n",
        );
        let sp_b = species_with_program_and_faction(
            "B",
            "F2",
            100,
            1000,
            "say(enemy_dist)\nwait(1)\ngoto start\n",
        );
        world.place_creature(&sp_a, 0, 0).expect("place A");
        world.place_creature(&sp_b, 3, 0).expect("place B");
        world
    }

    let world1 = make_world();
    let mut sim1 = Simulation::new(world1);
    sim1.set_rng_seed(42);
    let events1: Vec<Event> = (0..10).flat_map(|_| sim1.step_tick()).collect();
    let said1: Vec<String> = events1
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    let world2 = make_world();
    let mut sim2 = Simulation::new(world2);
    sim2.set_rng_seed(42);
    let events2: Vec<Event> = (0..10).flat_map(|_| sim2.step_tick()).collect();
    let said2: Vec<String> = events2
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(
        said1, said2,
        "same seed produces same said history:\n{said1:?}\n{said2:?}"
    );
}

#[test]
fn test_simulation_with_injected_rng_is_deterministic() {
    // AC3: An injected TestRng controls shuffle (turn order) and
    // next_bounded (random perception). Two runs with the same TestRng
    // produce the same history.
    // TestRng with fixed values: random returns 0, 1, 2 in sequence.
    // The creature says random each turn, so the TestRng controls the output.
    fn make_world() -> World {
        let map = Map {
            name: "test".to_string(),
            rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
        };
        let terrain = Terrain {
            name: "Grass".to_string(),
            character: '.',
            resource_max: 100,
            regen: 2,
            reserve: 100,
        };
        let mut world = World::from_map(&map, &[terrain], &[]);
        assert!(world.diagnostics.is_empty());
        let sp =
            species_with_program("Walker", 100, 1000, "say(random)\nwait(1)\ngoto start\n");
        let _id = world.place_creature(&sp, 0, 0).expect("place walker");
        world
    }

    // TestRng with fixed values: random returns 0, 1, 2 in sequence.
    let mut world1 = make_world();
    world1.rng = RefCell::new(Box::new(TestRng::new(vec![0, 1, 2])));
    let mut sim1 = Simulation::new(world1);
    let events1 = run_ticks(&mut sim1, 10);

    let mut world2 = make_world();
    world2.rng = RefCell::new(Box::new(TestRng::new(vec![0, 1, 2])));
    let mut sim2 = Simulation::new(world2);
    let events2 = run_ticks(&mut sim2, 10);

    assert_eq!(
        events1, events2,
        "same injected TestRng produces same history:\n{events1:?}\n{events2:?}"
    );

    let said1: Vec<&String> = events1
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(said1.iter().any(|t| **t == "0"), "says random=0, got: {said1:?}");
    assert!(said1.iter().any(|t| **t == "1"), "says random=1, got: {said1:?}");
    assert!(said1.iter().any(|t| **t == "2"), "says random=2, got: {said1:?}");

    // A different TestRng produces a different history (controls random).
    let mut world3 = make_world();
    world3.rng = RefCell::new(Box::new(TestRng::new(vec![5, 5, 5])));
    let mut sim3 = Simulation::new(world3);
    let events3 = run_ticks(&mut sim3, 10);

    let said3: Vec<&String> = events3
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said3.iter().all(|t| **t == "5"),
        "says random=5 always, got: {said3:?}"
    );
    assert_ne!(
        events1, events3,
        "different injected TestRng produces different history"
    );
}
