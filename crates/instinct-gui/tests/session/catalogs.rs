//! T1-T4: headless catalog loading over real fixtures.

use std::fs;

use instinct_gui::app::log::LogEntryKind;

use crate::common::examples_root;
use crate::common::LoadedSession;

/// Builds a minimal temp root with terrains, objects, maps and creatures folders.
/// Returns the temp root path. The caller must remove it.
fn temp_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("instinct_gui_test_{name}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("terrains")).expect("create terrains");
    fs::create_dir_all(root.join("objects")).expect("create objects");
    fs::create_dir_all(root.join("maps")).expect("create maps");
    fs::create_dir_all(root.join("creatures")).expect("create creatures");
    root
}

fn write_grass(root: &std::path::Path) {
    fs::write(
        root.join("terrains/grass.te"),
        "terrain Grass\nchar .\nresource_max 100\nregen 2\n",
    )
    .expect("write grass.te");
}

fn write_rock(root: &std::path::Path) {
    fs::write(
        root.join("objects/rock.ob"),
        "object Rock\nchar #\nresource_max 100\n",
    )
    .expect("write rock.ob");
}

fn load_diagnostics(session: &LoadedSession) -> Vec<String> {
    session
        .log_entries()
        .iter()
        .filter(|e| e.kind == LogEntryKind::Load)
        .map(|e| e.line.clone())
        .collect()
}

/// T1: `examples/` loads headless → catálogos no vacíos (mapa "Plain with rocks";
/// especies Dwarf, Ent, Hobbit, Uruk) y 0 diagnósticos de carga.
#[test]
fn t1_examples_loads_catalogs_without_diagnostics() {
    let session = LoadedSession::new(&examples_root());

    let map_names: Vec<&str> = session.maps().iter().map(|m| m.name.as_str()).collect();
    assert!(
        map_names.contains(&"Plain with rocks"),
        "maps must contain 'Plain with rocks', got {map_names:?}"
    );

    let species_names: Vec<&str> = session.species().iter().map(|s| s.name.as_str()).collect();
    for expected in ["Dwarf", "Ent", "Hobbit", "Uruk"] {
        assert!(
            species_names.contains(&expected),
            "species must contain '{expected}', got {species_names:?}"
        );
    }

    let diagnostics = load_diagnostics(&session);
    assert!(
        diagnostics.is_empty(),
        "examples must produce 0 load diagnostics, got {diagnostics:?}"
    );
}

/// T2: mapa con carácter desconocido → diagnóstico con línea; resto disponible;
/// el registro lo muestra.
#[test]
fn t2_unknown_character_produces_diagnostic_and_line() {
    let root = temp_root("t2");
    write_grass(&root);
    write_rock(&root);

    fs::write(root.join("maps/good.map"), "Good Map\n....\n....\n").expect("write good.map");
    fs::write(root.join("maps/bad.map"), "Bad Map\n...x......\n....\n").expect("write bad.map");

    let session = LoadedSession::new(&root);

    // Both maps are kept by the core; the bad one carries a diagnostic.
    let map_names: Vec<&str> = session.maps().iter().map(|m| m.name.as_str()).collect();
    assert!(
        map_names.contains(&"Good Map"),
        "good map must be available, got {map_names:?}"
    );
    assert!(
        map_names.contains(&"Bad Map"),
        "bad map must be available, got {map_names:?}"
    );

    let diagnostics = load_diagnostics(&session);
    assert!(
        diagnostics
            .iter()
            .any(|d| d.contains('x') && d.contains("unknown character")),
        "diagnostic must name the unknown character, got {diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|d| d.contains(":3:")),
        "diagnostic must carry the line number, got {diagnostics:?}"
    );

    let _ = fs::remove_dir_all(&root);
}

/// T3: sin mapas → `can_prepare()` falso y registro con "no maps".
#[test]
fn t3_no_maps_cannot_prepare() {
    let root = temp_root("t3");
    write_grass(&root);
    write_rock(&root);

    // A valid species so only maps are missing.
    fs::write(
        root.join("creatures/dwarf.ins"),
        "creature Dwarf\nfaction erebor\nhealth 100\nvision 3\nlifespan 600\nstart:\nwait(1)\n",
    )
    .expect("write dwarf.ins");

    let session = LoadedSession::new(&root);

    assert!(!session.can_prepare(), "without maps cannot prepare");
    assert!(session.maps().is_empty(), "no maps loaded");

    let diagnostics = load_diagnostics(&session);
    assert!(
        diagnostics.iter().any(|d| d.contains("no maps")),
        "log must report 'no maps', got {diagnostics:?}"
    );

    let _ = fs::remove_dir_all(&root);
}

/// T4: sin especies → `can_prepare()` falso y registro con "no species".
#[test]
fn t4_no_species_cannot_prepare() {
    let root = temp_root("t4");
    write_grass(&root);
    write_rock(&root);

    fs::write(
        root.join("maps/plain.map"),
        "Plain with rocks\n....\n....\n",
    )
    .expect("write plain.map");

    let session = LoadedSession::new(&root);

    assert!(!session.can_prepare(), "without species cannot prepare");
    assert!(session.species().is_empty(), "no species loaded");

    let diagnostics = load_diagnostics(&session);
    assert!(
        diagnostics.iter().any(|d| d.contains("no species")),
        "log must report 'no species', got {diagnostics:?}"
    );

    let _ = fs::remove_dir_all(&root);
}
