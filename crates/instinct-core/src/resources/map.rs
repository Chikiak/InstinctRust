//! Loading of a single map (`.map`).

use crate::diagnostic::Diagnostic;
use crate::resources::keys::LoadResult;

/// A loaded map: its name and its rows of characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub name: String,
    pub rows: Vec<Vec<char>>,
}

/// Loads a map from its in-memory content. There are no comments in maps.
/// `valid_characters` are the `char` declared by terrains and objects;
/// any other character produces a diagnostic with the line of its row
/// without preventing the map or the others from loading.
pub fn load_map(content: &str, file: &str, valid_characters: &[char]) -> LoadResult<Map> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            "the map must have at least one row (the name)".to_string(),
        ));
        return (None, diagnostics);
    }

    let name = lines[0].to_string();
    let rows_str = &lines[1..];

    if rows_str.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            2,
            "the map must have at least one row of cells".to_string(),
        ));
        return (None, diagnostics);
    }

    let mut rows: Vec<Vec<char>> = Vec::new();
    let mut expected_length: Option<usize> = None;

    for (idx, row) in rows_str.iter().enumerate() {
        let number = idx + 2; // line 1 is the name
        let chars: Vec<char> = row.chars().collect();
        if let Some(l) = expected_length {
            if chars.len() != l {
                diagnostics.push(Diagnostic::new(
                    file,
                    number,
                    format!(
                        "the row has {} characters, {} were expected",
                        chars.len(),
                        l
                    ),
                ));
                continue;
            }
        } else {
            expected_length = Some(chars.len());
        }
        for c in &chars {
            if !valid_characters.contains(c) {
                diagnostics.push(Diagnostic::new(
                    file,
                    number,
                    format!("unknown character in the map: '{c}'"),
                ));
            }
        }
        rows.push(chars);
    }

    if rows.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            2,
            "the map must have at least one row of cells".to_string(),
        ));
        return (None, diagnostics);
    }

    (Some(Map { name, rows }), diagnostics)
}
