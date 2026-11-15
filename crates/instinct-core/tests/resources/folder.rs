//! Folder loading tests for the `resources` module.
use super::common::*;

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
