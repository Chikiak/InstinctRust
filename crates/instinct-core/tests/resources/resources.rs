//! Integration tests for `load_resources` in the `resources` module.
use super::common::*;

/// AC8: A bad `.ins` inside `creatures/` does not stop loading the other
/// creatures nor the maps.
#[test]
fn test_bad_creature_does_not_stop_loading() {
    let temp = std::env::temp_dir().join("instinct_test_bad_creature");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(temp.join("creatures")).expect("create creatures dir");
    fs::create_dir_all(temp.join("maps")).expect("create maps dir");

    // A valid creature.
    fs::write(
        temp.join("creatures/good.ins"),
        "creature Good\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait(1)\n",
    )
    .expect("write good.ins");

    // An invalid creature (missing `start:`).
    fs::write(
        temp.join("creatures/bad.ins"),
        "creature Bad\nfaction any\nhealth 10\nvision 3\nlifespan 100\n",
    )
    .expect("write bad.ins");

    // A map referencing the grass terrain char.
    fs::write(temp.join("maps/test.map"), "Test Map\n....\n....\n").expect("write test.map");

    let resources = load_resources(&temp);

    // The valid creature should still be loaded.
    assert!(
        resources.creatures.iter().any(|c| c.name == "Good"),
        "good creature should be loaded despite bad one, got: {:?}",
        resources.creatures
    );

    // The bad creature should not be registered.
    assert!(
        !resources.creatures.iter().any(|c| c.name == "Bad"),
        "bad creature should not be registered"
    );

    // Diagnostics should report the missing start label.
    assert!(
        resources
            .diagnostics
            .iter()
            .any(|d| d.message.contains("start")),
        "diagnostics should report missing start, got: {:?}",
        resources.diagnostics
    );

    // The map should still be loaded.
    assert!(
        resources.maps.iter().any(|m| m.name == "Test Map"),
        "map should be loaded despite bad creature, got: {:?}",
        resources.maps
    );

    let _ = fs::remove_dir_all(&temp);
}
