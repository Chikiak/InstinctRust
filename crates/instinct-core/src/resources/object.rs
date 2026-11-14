//! Loading of a single object (`.ob`).

use crate::diagnostic::Diagnostic;
use crate::resources::keys::{
    key_value, parse_integer, read_keys, validate_single_character, LoadResult,
};

/// A loaded object. Its reserve starts at `resource_max` and does not regenerate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    pub name: String,
    pub character: char,
    pub resource_max: i64,
    pub reserve: i64,
}

/// Loads an object from its in-memory content.
pub fn load_object(content: &str, file: &str) -> LoadResult<Object> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let keys = read_keys(content);

    let name = match key_value(&keys, "object") {
        Some((n, v)) => {
            if v.trim().is_empty() {
                diagnostics.push(Diagnostic::new(
                    file,
                    n,
                    "missing value for key 'object'".to_string(),
                ));
                return (None, diagnostics);
            }
            v.trim().to_string()
        }
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'object'".to_string(),
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

    if resource_max <= 0 {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            format!("'resource_max' must be greater than 0, got {resource_max}"),
        ));
        return (None, diagnostics);
    }

    (
        Some(Object {
            name,
            character,
            resource_max,
            reserve: resource_max,
        }),
        diagnostics,
    )
}
