//! Loading of a single terrain (`.te`).

use crate::diagnostic::Diagnostic;
use crate::resources::keys::{
    key_value, parse_integer, read_keys, validate_single_character, LoadResult,
};

/// A loaded terrain. Its reserve starts full (equal to `resource_max`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terrain {
    pub name: String,
    pub character: char,
    pub resource_max: i64,
    pub regen: i64,
    pub reserve: i64,
}

/// Loads a terrain from its in-memory content.
pub fn load_terrain(content: &str, file: &str) -> LoadResult<Terrain> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let keys = read_keys(content);

    let name = match key_value(&keys, "terrain") {
        Some((n, v)) => {
            if v.trim().is_empty() {
                diagnostics.push(Diagnostic::new(
                    file,
                    n,
                    "missing value for key 'terrain'".to_string(),
                ));
                return (None, diagnostics);
            }
            v.trim().to_string()
        }
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'terrain'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let character = match key_value(&keys, "char") {
        Some((n, v)) => match validate_single_character(n, v, file, &mut diagnostics) {
            Some(c) => c,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'char'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let resource_max = match key_value(&keys, "resource_max") {
        Some((n, v)) => match parse_integer(n, v, "resource_max", file, &mut diagnostics) {
            Some(v) => v,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'resource_max'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let regen = match key_value(&keys, "regen") {
        Some((n, v)) => match parse_integer(n, v, "regen", file, &mut diagnostics) {
            Some(v) => v,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'regen'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    if resource_max <= 0 {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            format!("'resource_max' must be greater than 0, got {resource_max}"),
        ));
        return (None, diagnostics);
    }
    if regen < 0 {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            format!("'regen' must be greater than or equal to 0, got {regen}"),
        ));
        return (None, diagnostics);
    }

    (
        Some(Terrain {
            name,
            character,
            resource_max,
            regen,
            reserve: resource_max,
        }),
        diagnostics,
    )
}
