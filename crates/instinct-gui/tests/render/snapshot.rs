//! Headless tests for `grid_from_snapshot` (no window, no eframe).
//! Prioridad de render por celda: criatura > objeto > terreno.

use instinct_core::world::{
    CellSnapshot, CreatureId, CreatureState, LiveCreature, ObjectInfo, TerrainInfo, WorldSnapshot,
};
use instinct_gui::render::{grid_from_snapshot, CellKind};

fn sintetico() -> WorldSnapshot {
    WorldSnapshot {
        columns: 3,
        rows: 2,
        ticks: 42,
        map_name: "Prueba".to_string(),
        cells: vec![
            vec![
                // (0,0): solo terreno Hierba '.'.
                CellSnapshot {
                    terrain_char: '.',
                    object_char: None,
                    creature: None,
                },
                // (1,0): terreno Hierba '.' + objeto Roca '#'.
                CellSnapshot {
                    terrain_char: '.',
                    object_char: Some('#'),
                    creature: None,
                },
                // (2,0): terreno + objeto + criatura Talker (tapa a ambos).
                CellSnapshot {
                    terrain_char: '.',
                    object_char: Some('#'),
                    creature: Some(CreatureId(0)),
                },
            ],
            vec![
                // Fila 1: solo terreno Agua '~'.
                CellSnapshot {
                    terrain_char: '~',
                    object_char: None,
                    creature: None,
                },
                CellSnapshot {
                    terrain_char: '~',
                    object_char: None,
                    creature: None,
                },
                CellSnapshot {
                    terrain_char: '~',
                    object_char: None,
                    creature: None,
                },
            ],
        ],
        creatures: vec![LiveCreature {
            id: CreatureId(0),
            species_name: "Talker".to_string(),
            faction: "elfos".to_string(),
            x: 2,
            y: 0,
            health: 10,
            age: 1,
            lifespan: 100,
            state: CreatureState::Awake,
            pc: 0,
        }],
        objects: vec![ObjectInfo {
            index: 0,
            name: "Roca".to_string(),
            char: '#',
            reserve: 0,
        }],
        terrains: vec![
            TerrainInfo {
                name: "Hierba".to_string(),
                char: '.',
                reserve: 0,
                resource_max: 0,
            },
            TerrainInfo {
                name: "Agua".to_string(),
                char: '~',
                reserve: 0,
                resource_max: 0,
            },
        ],
        events: Vec::new(),
    }
}

#[test]
fn metadatos_del_snapshot_llegan_al_grid_view() {
    let grid = grid_from_snapshot(&sintetico());
    assert_eq!(grid.columns, 3);
    assert_eq!(grid.rows, 2);
    assert_eq!(grid.ticks, 42);
    assert_eq!(grid.map_name, "Prueba");
}

#[test]
fn celda_solo_terreno_usa_terreno_char_y_info_de_catalogo() {
    let grid = grid_from_snapshot(&sintetico());
    let c = &grid.cells[0][0];
    assert_eq!(c.symbol, '.');
    assert_eq!(c.label, "Hierba");
    assert_eq!(c.kind, CellKind::Terrain);
    assert_eq!(
        c.color,
        instinct_gui::render::Rgb {
            r: 120,
            g: 180,
            b: 120
        }
    );
}

#[test]
fn celda_con_objeto_sin_criatura_muestra_objeto() {
    let grid = grid_from_snapshot(&sintetico());
    let c = &grid.cells[0][1];
    assert_eq!(c.symbol, '#');
    assert_eq!(c.label, "Roca");
    assert_eq!(c.kind, CellKind::Object);
    assert_eq!(
        c.color,
        instinct_gui::render::Rgb {
            r: 200,
            g: 200,
            b: 200
        }
    );
}

#[test]
fn criatura_tapa_objeto_y_terreno_de_la_misma_celda() {
    let grid = grid_from_snapshot(&sintetico());
    let c = &grid.cells[0][2];
    assert_eq!(c.symbol, 'T');
    assert_eq!(c.label, "Talker (elfos)");
    assert_eq!(c.kind, CellKind::Creature);
    assert_eq!(
        c.color,
        instinct_gui::render::Rgb {
            r: 220,
            g: 60,
            b: 60
        }
    );
}

#[test]
fn fila_solo_terreno_todas_celdas_iguales() {
    let grid = grid_from_snapshot(&sintetico());
    assert!(
        grid.cells[1].len() >= 3,
        "la fila 1 debe tener al menos 3 celdas para compararlas: tiene {}",
        grid.cells[1].len()
    );
    for (x, c) in grid.cells[1].iter().enumerate().take(3) {
        assert_eq!(c.symbol, '~');
        assert_eq!(c.label, "Agua");
        assert_eq!(c.kind, CellKind::Terrain);
        assert_eq!(c.x, x);
        assert_eq!(c.y, 1);
    }
}
