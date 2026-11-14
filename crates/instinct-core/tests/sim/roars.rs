//! Roars scenario tests, moved from `tests/sim_tests.rs`.
//! Comments translated to English; test logic is identical to the original.

use super::common::*;

use instinct_core::event::{Event, Pos};
use instinct_core::sim::Simulation;

#[test]
fn test_roar_rotates_to_previous_tick() {
    // A roar emitted in the current tick becomes "from the previous tick" at
    // the end of the tick (sim.rs rotation), so F2 can read it with
    // roars_near/last_roar on the following tick.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_no_program("Walker", 100, 1000);
    let id = world.place_creature(&sp, 1, 1).expect("place walker");
    let mut sim = Simulation::new(world);

    sim.world.add_roar(
        id,
        "Walker".to_string(),
        Pos { x: 1, y: 1 },
        "hola".to_string(),
    );
    assert_eq!(sim.world.roars_current.len(), 1);
    assert!(sim.world.roars_previous.is_empty());
    assert_eq!(sim.world.roars_current[0].tick, sim.world.ticks);

    let _events = sim.step_tick();

    // After the tick: rotated to previous, current empty.
    assert!(sim.world.roars_current.is_empty(), "current was cleared");
    assert_eq!(sim.world.roars_previous.len(), 1, "they moved to previous");
    let stored = &sim.world.roars_previous[0];
    assert_eq!(
        stored.tick, 0,
        "roar tick (current tick - 1 after the advance)"
    );
    assert_eq!(stored.text, "hola");
    assert_eq!(stored.pos, Pos { x: 1, y: 1 });
    assert_eq!(stored.species, "Walker");

    // A second step without new roars empties previous: a roar is only
    // available on the tick after its emission, it does not persist forever.
    let _events = sim.step_tick();
    assert!(
        sim.world.roars_previous.is_empty(),
        "no roars from the previous tick"
    );
    assert!(sim.world.roars_current.is_empty());
}

#[test]
fn test_roars_near_and_last_roar() {
    // Creature E (emitter) roars in tick 0. Creature L (listener) is within
    // vision and should hear it in tick 1.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    let sp_e = species_with_program_and_faction(
        "E",
        "A",
        100,
        1000,
        "roar(\"hola mundo\")\nwait(100)\ngoto start\n",
    );
    let _id_e = world.place_creature(&sp_e, 0, 0).expect("place emitter");

    let sp_l = species_with_program_and_faction(
        "L",
        "A", // same faction so E is an ally, but we test roars, not factions
        100,
        1000,
        "say(roars_near)\nsay(last_roar)\nmove(0, 0, 0)\ngoto start\n",
    );
    let _id_l = world
        .place_creature(&sp_l, 1, 0)
        .expect("place listener at distance 1");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);

    // Tick 0: E roars. L hears nothing yet (roar is in roars_current).
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    // In tick 0, L hasn't heard anything yet (the roar goes to previous at end of tick).
    assert!(
        said.iter().any(|t| **t == "0"),
        "L hears 0 roars in tick 0, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| t.is_empty()),
        "L hears empty last_roar in tick 0, got: {said:?}"
    );

    // Tick 1: E's roar is now in roars_previous. L should hear it.
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "1"),
        "L hears 1 roar in tick 1, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "hola mundo"),
        "L hears last_roar = 'hola mundo', got: {said:?}"
    );
}

#[test]
fn test_roar_is_heard_one_tick_only() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    // E roars once then sleeps; L roars every tick and reports what it
    // hears. E's roar must be heard by L in tick 1 (one tick after
    // emission) and not in tick 2. L must never hear its own roars.
    let sp_e = species_with_program_and_faction(
        "E",
        "A",
        100,
        1000,
        "roar(\"hola\")\nwait(100)\ngoto start\n",
    );
    let _id_e = world
        .place_creature(&sp_e, 0, 0)
        .expect("place emitter at (0,0)");

    let sp_l = species_with_program_and_faction(
        "L",
        "A",
        100,
        1000,
        "roar(\"propio\")\nsay(roars_near)\nsay(last_roar)\ngoto start\n",
    );
    let _id_l = world
        .place_creature(&sp_l, 1, 0)
        .expect("place listener at (1,0)");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(3);

    // Tick 0: E roars "hola", L roars "propio". No says yet (roar ends turn).
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.is_empty(),
        "no says in tick 0 (roar ends turn), got: {said:?}"
    );

    // Tick 1: E sleeps. L says roars_near=1 (hears E's roar, not its own),
    // last_roar="hola". Then L roars again.
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "1"),
        "L hears 1 roar in tick 1 (E's roar, own excluded), got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "hola"),
        "L hears last_roar='hola' in tick 1, got: {said:?}"
    );

    // Tick 2: E sleeps. E's roar from tick 0 rotated out. L's own roar from
    // tick 1 is in previous but excluded. roars_near=0, last_roar="".
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "0"),
        "L hears 0 roars in tick 2 (E's roar expired, own excluded), got: {said:?}"
    );
    assert!(
        said.iter().any(|t| t.is_empty()),
        "L hears empty last_roar in tick 2, got: {said:?}"
    );
}
