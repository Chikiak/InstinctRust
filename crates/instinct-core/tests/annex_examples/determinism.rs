//! Determinism (RNG regression) tests for the annex examples.

use super::common::*;
use instinct_core::rng::Rng;
use instinct_core::sim::Simulation;

#[test]
fn test_annex_seed7_sequence_unchanged() {
    // RNG regression: Rng::new(7) produces the same next_bounded and shuffle
    // sequence as before the issue (values captured in the base version).
    let mut rng = Rng::new(7);
    assert_eq!(rng.next_bounded(99), 87);
    assert_eq!(rng.next_bounded(99), 4);
    assert_eq!(rng.next_bounded(99), 46);
    assert_eq!(rng.next_bounded(99), 3);
    assert_eq!(rng.next_bounded(99), 74);
    assert_eq!(rng.next_bounded(99), 5);
    assert_eq!(rng.next_bounded(99), 98);
    assert_eq!(rng.next_bounded(99), 82);

    // Shuffle with seed 7: indices [0..6] -> [2, 4, 1, 3, 0, 5].
    let mut order: Vec<usize> = (0..6).collect();
    rng.shuffle(&mut order);
    assert_eq!(order, vec![2, 4, 1, 3, 0, 5]);

    // The four examples with --seed 7 are deterministic: two identical runs.
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
            "seed 7 deterministic for {}:\n{events1:?}\n{events2:?}",
            name
        );
    }
}
