//! Draw error tests: test_draw_no_orphan_panic, test_draw_short_catalog_errors.
//! Bodies copied verbatim from tests/world_tests.rs; Spanish comments
//! translated to concise English. Assertion strings untouched.

use super::common::*;

#[test]
fn test_draw_no_orphan_panic() {
    let mut world = empty_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place Walker");
    // Orphan: the cell points to an id whose slot was freed; draw must
    // report it as an error, never panic.
    world.content[1][1] = CellContent::Creature(id);
    world.creatures[id.0] = None;

    let result = world.draw();
    assert!(
        result.is_err(),
        "draw must fail with orphan creature, no panic"
    );
    let err = result.unwrap_err();
    assert_eq!(err.file, "world");
    assert!(
        err.message.contains(&id.0.to_string()),
        "error names the orphan id: {err:?}"
    );
}

#[test]
fn test_draw_short_catalog_errors() {
    let mut world = empty_world();
    // content[0][0] references object 5, but the catalog has only 1 object.
    world.content[0][0] = CellContent::Object(5);

    let result = world.draw();
    assert!(result.is_err(), "draw must fail with short object catalog");
    let err = result.unwrap_err();
    assert!(err.message.contains("5"), "error names the index: {err:?}");

    // Short terrain catalog: the cell's terrain does not exist (the world has 1 terrain).
    let mut world2 = empty_world();
    world2.terrain[0][0] = 9;
    let result2 = world2.draw();
    assert!(
        result2.is_err(),
        "draw must fail with short terrain catalog"
    );
    let err2 = result2.unwrap_err();
    assert!(
        err2.message.contains("9"),
        "error names the index: {err2:?}"
    );
}
