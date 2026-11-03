//! Unit tests for the `resources` module.

use instinct_core::resources::{
    load_map, load_maps_folder, load_object, load_objects_folder, load_resources, load_terrain,
    load_terrains_folder,
};
use std::fs;
use std::path::Path;

#[test]
fn test_load_terrain_example() {
    let content = "terrain Grass\nchar .\nresource_max 100\nregen 2\n";
    let (result, d) = load_terrain(content, "grass.te");
    assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
    let t = result.expect("must load the terrain");
    assert_eq!(t.name, "Grass");
    assert_eq!(t.character, '.');
    assert_eq!(t.resource_max, 100);
    assert_eq!(t.regen, 2);
    assert_eq!(t.reserve, 100);
}

#[test]
fn test_load_object_example() {
    let content = "object Rock\nchar #\nresource_max 100\n";
    let (result, d) = load_object(content, "rock.ob");
    assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
    let o = result.expect("must load the object");
    assert_eq!(o.name, "Rock");
    assert_eq!(o.character, '#');
    assert_eq!(o.resource_max, 100);
    assert_eq!(o.reserve, 100);
}

#[test]
fn test_char_hash_is_not_a_comment() {
    // A line starting with '#' is a comment, but 'char #' is not.
    let content = "object Rock\nchar #\nresource_max 100\n";
    let (result, d) = load_object(content, "rock.ob");
    assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
    let o = result.expect("must load the object");
    assert_eq!(o.character, '#');
}

#[test]
fn test_load_map_example() {
    // Map of 20 columns x 7 rows with rocks at (3,1) and (4,1).
    let mut rows: Vec<Vec<char>> = Vec::new();
    for _ in 0..7 {
        rows.push(vec!['.'; 20]);
    }
    rows[1][3] = '#';
    rows[1][4] = '#';
    // Additional rocks from the example to fill the map.
    rows[2][3] = '#';
    rows[2][4] = '#';
    rows[2][9] = '#';
    rows[3][10] = '#';
    rows[3][15] = '#';
    rows[3][16] = '#';
    rows[3][17] = '#';
    rows[4][17] = '#';
    rows[5][1] = '#';
    rows[6][1] = '#';
    rows[6][10] = '#';
    rows[6][11] = '#';

    let content = "Plain with rocks\n".to_owned()
        + &rows
            .iter()
            .map(|r| r.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
        + "\n";
    let (result, d) = load_map(&content, "plain.map", &['.', '#']);
    assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
    let m = result.expect("must load the map");
    assert_eq!(m.name, "Plain with rocks");
    assert_eq!(m.rows.len(), 7);
    assert_eq!(m.rows[0].len(), 20);
    assert_eq!(m.rows[1][3], '#');
    assert_eq!(m.rows[1][4], '#');
    for (i, row) in m.rows.iter().enumerate() {
        assert_eq!(row.len(), 20, "row {i} does not have 20 columns");
    }
}

#[test]
fn test_map_unknown_char_reports_line() {
    // Only '.' is a valid character; 'x' is not. It is on line 3
    // (row index 1 -> line 3: 1 name + 1 row + 1).
    let content = "Rare map\n..........\n...x......\n";
    let (_result, d) = load_map(content, "rare.map", &['.']);
    assert!(!d.is_empty(), "there must be a diagnostic");
    assert!(
        d.iter()
            .any(|diag| diag.line == 3 && diag.message.contains('x')),
        "diagnostic on line 3 naming 'x', got: {d:?}"
    );
}

#[test]
fn test_map_ragged_row_reports_line() {
    // Second row shorter: line 3.
    let content = "Irregular map\n..........\n....\n..........\n";
    let (_result, d) = load_map(content, "irregular.map", &['.']);
    assert!(!d.is_empty(), "there must be a diagnostic");
    assert!(
        d.iter().any(|diag| diag.line == 3),
        "diagnostic on line 3, got: {d:?}"
    );
}

#[test]
fn test_resource_validation_diagnostics() {
    // Missing key
    let (_, d) = load_terrain("terrain X\nchar .\nregen 2\n", "missing.ob");
    assert!(
        d.iter().any(|diag| diag.message.contains("resource_max")),
        "missing resource_max: {d:?}"
    );
    // Non-numeric value
    let (_, d) = load_object("object X\nchar #\nresource_max abc\n", "number.ob");
    assert!(
        d.iter().any(|diag| diag.message.contains("resource_max")),
        "non-numeric value: {d:?}"
    );
    // char with more than one character
    let (_, d) = load_terrain("terrain X\nchar ab\nresource_max 10\nregen 1\n", "char.ob");
    assert!(
        d.iter().any(|diag| diag.message.contains("char")),
        "multi-char: {d:?}"
    );
    // resource_max <= 0
    let (_, d) = load_terrain("terrain X\nchar .\nresource_max 0\nregen 1\n", "max.ob");
    assert!(!d.is_empty(), "resource_max 0 must fail: {d:?}");
    // regen < 0
    let (_, d) = load_terrain("terrain X\nchar .\nresource_max 10\nregen -1\n", "regen.ob");
    assert!(!d.is_empty(), "regen < 0 must fail: {d:?}");
}

#[test]
fn test_missing_directory_is_empty() {
    let (_t, dt) = load_terrains_folder(Path::new("no_existe/terrains"));
    let (_o, do_) = load_objects_folder(Path::new("no_existe/objects"), &[]);
    let (_m, dm) = load_maps_folder(Path::new("no_existe/maps"), &[], &[]);
    assert!(dt.is_empty(), "missing folder without diagnostics: {dt:?}");
    assert!(
        do_.is_empty(),
        "missing folder without diagnostics: {do_:?}"
    );
    assert!(dm.is_empty(), "missing folder without diagnostics: {dm:?}");
}

#[test]
fn test_repeated_character_discards_file() {
    // Two objects with the same char '#': the first is kept, the second
    // is discarded with a diagnostic naming the char.
    let temp = std::env::temp_dir().join("instinct_test_repeated_character");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).expect("create temp dir");
    fs::write(temp.join("a.ob"), "object Rock\nchar #\nresource_max 100\n").expect("write a.ob");
    fs::write(temp.join("b.ob"), "object Otro\nchar #\nresource_max 50\n").expect("write b.ob");
    let (objects, d) = load_objects_folder(&temp, &[]);
    assert_eq!(objects.len(), 1, "only the first is kept");
    assert_eq!(objects[0].name, "Rock");
    assert!(
        d.iter().any(|diag| diag.message.contains('#')),
        "diagnostic names the repeated char: {d:?}"
    );
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn test_alphabetical_order_first_terrain() {
    // Two terrains: "b.te" declares "B", "a.te" declares "A". The first in
    // alphabetical file order is "a.te".
    let (result_a, _) = load_terrain("terrain A\nchar a\nresource_max 5\nregen 1\n", "a.te");
    let (_result_b, _) = load_terrain("terrain B\nchar b\nresource_max 5\nregen 1\n", "b.te");
    let ta = result_a.expect("a.te");
    assert_eq!(ta.name, "A");
    assert_eq!(ta.character, 'a');
}

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
