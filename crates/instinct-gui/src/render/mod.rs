//! Pure grid geometry: cell <-> pixel conversions.
//! No knowledge of eframe/egui; fully testable headless.

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
