//! Pruebas de escenario F6-06 (núcleo): carga tolerante del banco de recursos
//! en `examples/traps/`.
//!
//! `examples/traps/` contiene la criatura valida `Valid` y los 11 archivos de
//! trampa de compilacion. Cada trampa se descarta con un diagnostico sin
//! abortar la carga de los demas recursos (terrains, objects, maps).

use instinct_core::resources::load_resources;
use std::path::Path;

/// Especies que declaran los 11 archivos de trampa (cada uno se descarta al
/// cargar, por lo que ninguna debe aparecer entre las especies cargadas).
const TRAP_SPECIES: &[&str] = &[
    "BadLine",
    "UnknownLabel",
    "DupLabel",
    "MissingStart",
    "BadExpr",
    "AssignPerception",
    "AssignConstant",
    "UnknownAction",
    "UnknownFunction",
    "WrongArity",
];

/// Carpeta de recursos de trampas: `examples/traps/`, relativa al manifiesto
/// de este crate.
fn traps_dir() -> String {
    format!("{}/../../examples/traps", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn load_traps_resources_tolerates_invalid_creatures() {
    let resources = load_resources(Path::new(&traps_dir()));

    // Las trampas (y los mapas mal formados) producen diagnoticos pero no
    // detienen la carga de los demas recursos.
    assert!(
        !resources.diagnostics.is_empty(),
        "deben haber diagnoticos de las trampas, got {:?}",
        resources.diagnostics
    );

    // La criatura valida se conserva.
    assert!(
        resources.creatures.iter().any(|s| s.name == "Valid"),
        "Valid debe estar cargada, especies: {:?}",
        creature_names(&resources)
    );

    // Ninguna especie de trampa se conserva (todas fueron descartadas).
    for especie in TRAP_SPECIES {
        assert!(
            !resources.creatures.iter().any(|s| s.name == *especie),
            "{especie} no debe estar cargada (trampa descartada)"
        );
    }

    // Recursos validos de la carpeta traps estan presentes.
    assert!(
        resources.terrains.iter().any(|t| t.name == "Grass"),
        "terrains debe contener Grass"
    );
    assert!(
        resources.objects.iter().any(|o| o.name == "Rock"),
        "objects debe contener Rock"
    );
    assert!(
        resources.maps.iter().any(|m| m.name == "Arena"),
        "maps debe contener Arena"
    );
}

fn creature_names(resources: &instinct_core::resources::LoadedResources) -> Vec<String> {
    resources.creatures.iter().map(|s| s.name.clone()).collect()
}
