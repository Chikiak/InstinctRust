use instinct_core::resources::{Map, Terrain};
use instinct_core::world::World;
use std::rc::Rc;

/// ASCII grid rows printed after the `== tick N ==` header,
/// up to the next tick header (or end of output).
pub fn rows_after_tick(stdout: &str, tick: u64) -> Vec<String> {
    let marker = format!("== tick {tick} ==");
    let mut rows: Vec<String> = Vec::new();
    let mut found = false;
    for line in stdout.lines() {
        if line == marker {
            found = true;
            continue;
        }
        if found {
            if line.starts_with("== tick ") {
                break;
            }
            rows.push(line.to_string());
        }
    }
    rows
}

/// Minimal grass world (5x5) used to build the World by hand.
/// The world retains the catalogs (sole owner).
pub fn grass_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
        ],
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
    world
}

/// Places a creature with the given health (Species::new has health 0).
pub fn place_with_health(
    world: &mut World,
    x: usize,
    y: usize,
    health: i64,
) -> instinct_core::world::CreatureId {
    let species = Rc::new(instinct_core::species::Species::new("Walker"));
    let id = world
        .place_creature(&species, x, y)
        .expect("place creature");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = health;
        }
    }
    id
}

/// Returns the health of a living creature; fails if it does not exist.
pub fn creature_health(world: &World, id: instinct_core::world::CreatureId) -> i64 {
    world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id)
        .map(|c| c.health)
        .expect("creature exists")
}
