//! Header parsing and validation for `.ins` creature files.

use crate::diagnostic::Diagnostic;

/// Parsed header of a `.ins` file: the five required attributes.
#[derive(Debug, Clone)]
pub struct Header {
    pub species_name: String,
    pub faction: String,
    pub health: i64,
    pub vision: i64,
    pub lifespan: i64,
}

/// The five required header keys, in no particular order.
pub const HEADER_KEYS: &[&str] = &["creature", "faction", "health", "vision", "lifespan"];

/// Splits a header line `key value` into (key, value). Returns None when the
/// line is not a simple key-value pair.
pub fn split_header(line: &str) -> Option<(&str, &str)> {
    let idx = line.find(char::is_whitespace)?;
    let key = &line[..idx];
    let value = line[idx..].trim();
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some((key, value))
    }
}

/// Validates that a header value is an integer > 0 (vision >= 1). Pushes a
/// diagnostic naming the key when invalid.
///
/// The value is parsed exactly once (the caller already parsed it and passes
/// the result); an invalid integer or a value below the minimum produces a
/// single diagnostic on the value's line.
pub fn validate_header_value(
    key: &str,
    value: &str,
    line: usize,
    file: &str,
    parsed: Result<i64, std::num::ParseIntError>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match parsed {
        Ok(n) => {
            let min = 1;
            if n < min {
                diagnostics.push(Diagnostic::new(
                    file,
                    line,
                    format!("the value of '{key}' must be greater than 0, got {n}"),
                ));
            }
        }
        Err(_) => {
            diagnostics.push(Diagnostic::new(
                file,
                line,
                format!("the value of '{key}' must be an integer, got '{value}'"),
            ));
        }
    }
}
