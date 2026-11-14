//! Shared test helpers for the simulation scenario tests.
//! Extracted from `tests/sim_tests.rs` (lines 1-143): TestRng, terrain/object/
//! map builders, species constructors, and build_world.
//! Logic is identical to the original; comments were translated to English and
//! items are `pub` so sibling submodules can import them.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::perceptions::PerceptionCatalog;
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::rng::RngSource;
use instinct_core::species::Species;
use instinct_core::world::World;
use std::rc::Rc;

/// Deterministic RNG for tests: returns values from a fixed sequence.
#[derive(Debug, Clone)]
pub struct TestRng {
    values: Vec<i64>,
    index: usize,
}

impl TestRng {
    pub fn new(values: Vec<i64>) -> Self {
        TestRng { values, index: 0 }
    }
}

impl RngSource for TestRng {
    fn next_bounded(&mut self, max: i64) -> i64 {
        let v = self.values[self.index % self.values.len()];
        self.index += 1;
        if max < 0 {
            return 0;
        }
        let range = (max as u64).wrapping_add(1);
        (v as u64 % range) as i64
    }

    fn shuffle(&mut self, list: &mut [usize]) {
        let mut i = list.len();
        while i > 1 {
            i -= 1;
            let j = self.next_bounded(i as i64) as usize;
            list.swap(i, j);
        }
    }
}

pub fn grass_terrain() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

pub fn rock_object() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

pub fn empty_map() -> Map {
    Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    }
}

/// Compiles an `.ins` and returns the `Species` with its program wrapped in
/// `Rc` to share it through the world catalog (Flyweight).
pub fn species_with_program(name: &str, health: i64, lifespan: i64, body: &str) -> Rc<Species> {
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
    Rc::new(Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    ))
}

pub fn species_no_program(name: &str, health: i64, lifespan: i64) -> Rc<Species> {
    Rc::new(Species {
        name: name.to_string(),
        faction: "any".to_string(),
        health,
        vision: 3,
        lifespan,
        program: None,
    })
}

/// Compiles an `.ins` with the given faction and returns the `Species` with
/// its program wrapped in `Rc`. Used to test perceptions that read the world
/// (faction, totals).
pub fn species_with_program_and_faction(
    name: &str,
    faction: &str,
    health: i64,
    lifespan: i64,
    body: &str,
) -> Rc<Species> {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text = format!(
        "creature {name}\nfaction {faction}\nhealth {health}\nvision 3\nlifespan {lifespan}\nstart:\n{body}"
    );
    let result = compile::compile(&text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("header");
    Rc::new(Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    ))
}

pub fn build_world(map: &Map, terrains: &[Terrain], objects: &[Object]) -> World {
    let world = World::from_map(map, terrains, objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}
