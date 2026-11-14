//! Species tests: test_creatures_share_species_rc.
//! Bodies copied verbatim from tests/world_tests.rs; Spanish comments
//! translated to concise English. Assertion strings untouched.

use super::common::*;

#[test]
fn test_creatures_share_species_rc() {
    let mut world = empty_world();
    let grass = Rc::new(Species::new("Grass"));

    let id1 = world
        .place_creature(&grass, 0, 0)
        .expect("place first creature");
    let id2 = world
        .place_creature(&grass, 1, 1)
        .expect("place second creature");

    let c1 = world.creatures[id1.0].as_ref().expect("creature 1 alive");
    let c2 = world.creatures[id2.0].as_ref().expect("creature 2 alive");

    // Both creatures point to the same Species instance.
    assert!(
        Rc::ptr_eq(&c1.species, &c2.species),
        "creatures of the same species must share the Rc<Species>"
    );
    // And the world registered its own copy in the catalog with the same pointer.
    assert!(
        world
            .species
            .iter()
            .any(|owned| Rc::ptr_eq(owned, &c1.species)),
        "the world catalog owns the shared Rc<Species>"
    );
    // draw still shows the first letter of the shared species.
    let drawing = world.draw().unwrap();
    let lines: Vec<&str> = drawing.lines().collect();
    assert_eq!(lines[0], "G..");
    assert_eq!(lines[1], ".G.");
    assert_eq!(lines[2], "...");
}
