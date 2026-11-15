use super::common::TestRng;
use instinct_core::resources::{Map, Terrain};
use instinct_core::world::World;
use std::cell::RefCell;

#[test]
fn test_world_uses_rng_source_trait() {
    // AC2: World uses Box<dyn RngSource>; set_rng_seed wraps Rng::new(seed).
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 3], vec!['.'; 3], vec!['.'; 3]],
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

    // set_rng_seed(7) wraps Rng::new(7): same sequence as Rng::new(7).
    world.set_rng_seed(7);
    assert_eq!(world.rng.borrow_mut().next_bounded(99), 87);
    assert_eq!(world.rng.borrow_mut().next_bounded(99), 4);
    assert_eq!(world.rng.borrow_mut().next_bounded(99), 46);

    // Inject a TestRng: World accepts any Box<dyn RngSource>.
    world.rng = RefCell::new(Box::new(TestRng::new(vec![42])));
    assert_eq!(world.rng.borrow_mut().next_bounded(99), 42);
    assert_eq!(world.rng.borrow_mut().next_bounded(99), 42);
}
