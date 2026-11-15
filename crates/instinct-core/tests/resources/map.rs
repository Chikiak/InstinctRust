//! Map loading tests for the `resources` module.
use super::common::*;

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
