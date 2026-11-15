//! Creature using `see`/`name` test (AC8).

use super::common::*;

// ---------------------------------------------------------------------------
// AC8: creature using see/name runs without errors
// ---------------------------------------------------------------------------

#[test]
fn test_creature_uses_see_and_name() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_with_program(
        "Watcher",
        "A",
        100,
        1000,
        "say(see(1, 0))\nsay(name(1, 0))\nwait(1)\ngoto start\n",
    );
    let _id = world.place_creature(&sp, 0, 0).expect("place watcher");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);
    let events = sim.step_tick();

    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    // Both see and name should produce output without error.
    assert!(
        said.len() >= 2,
        "creature using see and name should say at least twice, got: {said:?}"
    );
    assert!(
        !said.iter().any(|t| t.is_empty()),
        "outputs should be non-empty, got: {said:?}"
    );
}
