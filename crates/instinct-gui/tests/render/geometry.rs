use instinct_gui::render::{
    cell_to_pixel, pixel_to_cell, Cell, CELL_SIZE_PX, GRID_COLS, GRID_ROWS, MARGIN_PX,
};

/// Origin of the drawable grid once the surrounding margin is added.
fn origin() -> (f32, f32) {
    (MARGIN_PX, MARGIN_PX)
}

#[test]
fn constants_are_consistent() {
    assert_eq!(CELL_SIZE_PX, 32.0);
    assert_eq!(MARGIN_PX, 8.0);
    assert_eq!(GRID_COLS, 16);
    assert_eq!(GRID_ROWS, 12);
    assert!((GRID_COLS as f32 * CELL_SIZE_PX) > 0.0);
    assert!((GRID_ROWS as f32 * CELL_SIZE_PX) > 0.0);
}

#[test]
fn cell_to_pixel_returns_top_left_of_cell_relative_to_grid_origin() {
    assert_eq!(cell_to_pixel(Cell { x: 0, y: 0 }), (0.0, 0.0));
    assert_eq!(cell_to_pixel(Cell { x: 1, y: 0 }), (CELL_SIZE_PX, 0.0));
    assert_eq!(cell_to_pixel(Cell { x: 0, y: 1 }), (0.0, CELL_SIZE_PX));
    assert_eq!(
        cell_to_pixel(Cell {
            x: GRID_COLS - 1,
            y: GRID_ROWS - 1,
        }),
        (
            (GRID_COLS - 1) as f32 * CELL_SIZE_PX,
            (GRID_ROWS - 1) as f32 * CELL_SIZE_PX
        )
    );
}

#[test]
fn pixel_to_cell_round_trip_through_cell_centers() {
    for y in 0..GRID_ROWS {
        for x in 0..GRID_COLS {
            let cell = Cell { x, y };
            let (px, py) = cell_to_pixel(cell);
            let center = (
                origin().0 + px + CELL_SIZE_PX / 2.0,
                origin().1 + py + CELL_SIZE_PX / 2.0,
            );
            assert_eq!(pixel_to_cell(center.0, center.1), Some(cell));
        }
    }
}

#[test]
fn pixel_to_cell_returns_none_in_margins_and_outside_grid() {
    // Inside the surrounding margin: not part of any cell.
    assert_eq!(pixel_to_cell(0.0, 0.0), None);
    assert_eq!(pixel_to_cell(-1.0, -1.0), None);
    assert_eq!(pixel_to_cell(MARGIN_PX / 2.0, MARGIN_PX / 2.0), None);
    assert_eq!(pixel_to_cell(0.0, MARGIN_PX + CELL_SIZE_PX / 2.0), None);
    assert_eq!(pixel_to_cell(MARGIN_PX + CELL_SIZE_PX / 2.0, 0.0), None);
    // Above the grid.
    assert_eq!(pixel_to_cell(CELL_SIZE_PX / 2.0, -0.5), None);
    // Beyond the right / bottom border.
    let too_far_x = MARGIN_PX + GRID_COLS as f32 * CELL_SIZE_PX + 1.0;
    assert_eq!(
        pixel_to_cell(too_far_x, origin().1 + CELL_SIZE_PX / 2.0),
        None
    );
    let too_far_y = MARGIN_PX + GRID_ROWS as f32 * CELL_SIZE_PX + 1.0;
    assert_eq!(
        pixel_to_cell(origin().0 + CELL_SIZE_PX / 2.0, too_far_y),
        None
    );
    // Exactly on the right / bottom border: outside the last cell.
    assert_eq!(
        pixel_to_cell(
            origin().0 + GRID_COLS as f32 * CELL_SIZE_PX,
            origin().1 + GRID_ROWS as f32 * CELL_SIZE_PX / 2.0
        ),
        None
    );
    assert_eq!(
        pixel_to_cell(
            origin().0 + GRID_COLS as f32 * CELL_SIZE_PX / 2.0,
            origin().1 + GRID_ROWS as f32 * CELL_SIZE_PX
        ),
        None
    );
}

#[test]
fn pixel_to_cell_floors_to_containing_cell() {
    let (px, py) = cell_to_pixel(Cell { x: 3, y: 4 });
    // Any point strictly inside the cell maps back to it (margins excluded).
    assert_eq!(
        pixel_to_cell(origin().0 + px + 1.0, origin().1 + py + 1.0),
        Some(Cell { x: 3, y: 4 })
    );
    assert_eq!(
        pixel_to_cell(
            origin().0 + px + CELL_SIZE_PX - 0.5,
            origin().1 + py + CELL_SIZE_PX - 0.5
        ),
        Some(Cell { x: 3, y: 4 })
    );
    // Exact top-left corner of a cell belongs to that cell.
    assert_eq!(
        pixel_to_cell(origin().0, origin().1),
        Some(Cell { x: 0, y: 0 })
    );
    // Cell boundary floors to the right/below cell (left-closed intervals).
    assert_eq!(
        pixel_to_cell(origin().0 + CELL_SIZE_PX, origin().1 + CELL_SIZE_PX / 2.0),
        Some(Cell { x: 1, y: 0 })
    );
}
