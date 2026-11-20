//! Pure grid geometry: cell <-> pixel conversions.
//! No knowledge of eframe/egui; fully testable headless.

pub mod roars;

use instinct_core::world::{CreatureId, WorldSnapshot};

pub use roars::roar_markers;

/// Size of one square cell in pixels.
pub const CELL_SIZE_PX: f32 = 32.0;
/// Margin (in pixels) around the grid that belongs to no cell.
pub const MARGIN_PX: f32 = 8.0;
/// Number of columns of the grid.
pub const GRID_COLS: usize = 16;
/// Number of rows of the grid.
pub const GRID_ROWS: usize = 12;

/// Total width of the grid area in pixels (margins excluded).
pub const GRID_WIDTH_PX: f32 = GRID_COLS as f32 * CELL_SIZE_PX;
/// Total height of the grid area in pixels (margins excluded).
pub const GRID_HEIGHT_PX: f32 = GRID_ROWS as f32 * CELL_SIZE_PX;

/// Cell of the grid, in (column, row) coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub x: usize,
    pub y: usize,
}

/// Top-left corner of `cell` relative to the grid origin (margins excluded).
pub fn cell_to_pixel(cell: Cell) -> (f32, f32) {
    (cell.x as f32 * CELL_SIZE_PX, cell.y as f32 * CELL_SIZE_PX)
}

/// Pixel coordinate -> cell.
///
/// The margin is subtracted first; points in the margin or outside the
/// grid yield `None`. The rest is floored to the containing cell.
pub fn pixel_to_cell(x: f32, y: f32) -> Option<Cell> {
    let (x, y) = (x - MARGIN_PX, y - MARGIN_PX);
    if x < 0.0 || y < 0.0 || x >= GRID_WIDTH_PX || y >= GRID_HEIGHT_PX {
        return None;
    }
    Some(Cell {
        x: (x / CELL_SIZE_PX).floor() as usize,
        y: (y / CELL_SIZE_PX).floor() as usize,
    })
}

/// Pure RGB color, independent of any GUI toolkit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Qué contenido se muestra en la celda. Prioridad de render:
/// `Creature` > `Object` > `Terrain`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellKind {
    Terrain,
    Object,
    Creature,
}

/// Una celda lista para pintar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellView {
    pub x: usize,
    pub y: usize,
    pub symbol: char,
    pub label: String,
    pub kind: CellKind,
    pub color: Rgb,
}

/// Una malla completa traducida de un `WorldSnapshot`, lista para pintar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GridView {
    pub columns: usize,
    pub rows: usize,
    pub ticks: u64,
    pub map_name: String,
    pub cells: Vec<Vec<CellView>>,
}

/// Traduce un `WorldSnapshot` de solo lectura en un `GridView`.
///
/// Regla por celda (`cells[y][x]`): criatura > objeto > terreno. Los
/// símbolos son los caracteres del DTO (criatura = inicial de `species_name`).
pub fn grid_from_snapshot(snapshot: &WorldSnapshot) -> GridView {
    let cells: Vec<Vec<CellView>> = (0..snapshot.rows)
        .map(|y| {
            (0..snapshot.columns)
                .map(|x| cell_view(snapshot, x, y))
                .collect()
        })
        .collect();

    GridView {
        columns: snapshot.columns,
        rows: snapshot.rows,
        ticks: snapshot.ticks,
        map_name: snapshot.map_name.clone(),
        cells,
    }
}

/// Render de una sola celda aplicando la prioridad criatura > objeto > terreno.
fn cell_view(snapshot: &WorldSnapshot, x: usize, y: usize) -> CellView {
    let cell = &snapshot.cells[y][x];
    if let Some(creature_id) = cell.creature {
        creature_view(snapshot, x, y, creature_id)
    } else if let Some(object_char) = cell.object_char {
        object_view(snapshot, x, y, object_char)
    } else {
        terrain_view(snapshot, x, y, cell.terrain_char)
    }
}

fn creature_view(snapshot: &WorldSnapshot, x: usize, y: usize, id: CreatureId) -> CellView {
    let creature = snapshot.creatures.iter().find(|c| c.id == id);
    let species_name = creature.map(|c| c.species_name.as_str()).unwrap_or("");
    let faction = creature.map(|c| c.faction.as_str()).unwrap_or("");
    let symbol = species_name.chars().next().unwrap_or('?');
    CellView {
        x,
        y,
        symbol,
        label: format!("{species_name} ({faction})"),
        kind: CellKind::Creature,
        color: Rgb {
            r: 220,
            g: 60,
            b: 60,
        },
    }
}

fn object_view(snapshot: &WorldSnapshot, x: usize, y: usize, object_char: char) -> CellView {
    let object = snapshot.objects.iter().find(|o| o.char == object_char);
    CellView {
        x,
        y,
        symbol: object_char,
        label: object
            .map(|o| o.name.as_str())
            .unwrap_or("objeto desconocido")
            .to_string(),
        kind: CellKind::Object,
        color: Rgb {
            r: 200,
            g: 200,
            b: 200,
        },
    }
}

fn terrain_view(snapshot: &WorldSnapshot, x: usize, y: usize, terrain_char: char) -> CellView {
    let terrain = snapshot.terrains.iter().find(|t| t.char == terrain_char);
    CellView {
        x,
        y,
        symbol: terrain_char,
        label: terrain
            .map(|t| t.name.as_str())
            .unwrap_or("terreno desconocido")
            .to_string(),
        kind: CellKind::Terrain,
        color: Rgb {
            r: 120,
            g: 180,
            b: 120,
        },
    }
}
