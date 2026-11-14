//! Dwarf example tests.

use super::common::*;
use instinct_core::event::{Event, Pos};
use instinct_core::resources::Object;
use instinct_core::sim::Simulation;

#[test]
fn test_dwarf_mines_adjacent_rock() {
    // AC4: Dwarf mines an adjacent rock.
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
