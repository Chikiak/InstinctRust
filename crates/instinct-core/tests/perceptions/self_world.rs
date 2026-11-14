//! Self and world perception tests.

use super::common::*;
use instinct_core::actions::Catalog;
use instinct_core::expr::parse;
use instinct_core::interp::InterpreterVisitor;
use instinct_core::perceptions::{CreatureView, Perception, PerceptionCatalog};
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::{CreatureContext, CreatureId, World};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_self_perceptions_report_species_data() {
    let mut world = empty_world();
    // The species is built with its intrinsic data and shared via Rc;
    // it is not mutated after placement.
    let species = Rc::new(Species {
        name: "Hobbit".to_string(),
        faction: "Comarca".to_string(),
        health: 0,
        vision: 2,
        lifespan: 0,
        program: None,
    });
    let id = world.place_creature(&species, 4, 2).expect("place hobbit");

    assert_eq!(
        eval(&world, id, "species").unwrap(),
        Value::Text("Hobbit".to_string())
    );
    assert_eq!(
        eval(&world, id, "faction").unwrap(),
        Value::Text("Comarca".to_string())
    );
    assert_eq!(unwrap_int(eval(&world, id, "vision").unwrap()), 2);
}

#[test]
fn test_terrain_here_tracks_reserve() {
    let mut world = empty_world(); // 5x5 Grass, reserve 100
    let id = place_walker(&mut world); // (4, 2)

    assert_eq!(
        eval(&world, id, "terrain_here").unwrap(),
        Value::Text("Grass".to_string())
    );
    assert_eq!(unwrap_int(eval(&world, id, "reserve_here").unwrap()), 100);

    // Remove the creature to damage the terrain under it without harming it:
    // damage_cell_content on a cell with a creature does not touch the terrain.
    world.remove_creature(id).expect("remove creature");
    world
        .damage_cell_content(4, 2, 30)
        .expect("damage terrain at (4,2)");
    assert_eq!(world.terrains[0].reserve, 70);

    // Re-place on the same cell (new id) and read the damaged reserve.
    let id2 = world
        .place_creature(&Rc::new(Species::new("Walker")), 4, 2)
        .expect("re-place creature");
    assert_eq!(unwrap_int(eval(&world, id2, "reserve_here").unwrap()), 70);

    // End of tick: regenerates +2 over the damaged reserve: 72 (capped at 100).
    world.end_of_tick_terrain_and_objects();
    assert_eq!(unwrap_int(eval(&world, id2, "reserve_here").unwrap()), 72);
}

#[test]
fn test_totals_count_world_excluding_self() {
    let mut world = empty_world(); // 5x5

    let faction_a = Rc::new(Species {
        name: "A".to_string(),
        faction: "A".to_string(),
        health: 0,
        vision: 0,
        lifespan: 0,
        program: None,
    });
    let id_self = world
        .place_creature(&faction_a, 0, 0)
        .expect("place self");

    let _id_ally = world
        .place_creature(&faction_a, 1, 0)
        .expect("place ally");

    let faction_b = Rc::new(Species {
        name: "B".to_string(),
        faction: "B".to_string(),
        health: 0,
        vision: 0,
        lifespan: 0,
        program: None,
    });
    let _id_enemy1 = world
        .place_creature(&faction_b, 2, 0)
        .expect("place enemy 1");
    let _id_enemy2 = world
        .place_creature(&faction_b, 3, 0)
        .expect("place enemy 2");

    assert_eq!(unwrap_int(eval(&world, id_self, "width").unwrap()), 5);
    assert_eq!(unwrap_int(eval(&world, id_self, "height").unwrap()), 3);
    // Counts the other 3 creatures, not itself.
    assert_eq!(
        unwrap_int(eval(&world, id_self, "total_creatures").unwrap()),
        3
    );
    // One ally of faction A (not counting itself).
    assert_eq!(
        unwrap_int(eval(&world, id_self, "total_allies").unwrap()),
        1
    );
    // Two enemies of faction B.
    assert_eq!(
        unwrap_int(eval(&world, id_self, "total_enemies").unwrap()),
        2
    );
}
