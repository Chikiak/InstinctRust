//! Header parsing and validation for `.ins` creature files.

use crate::diagnostic::Diagnostic;
use crate::program::Line;

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

/// Processes one line while still in the header phase (before `start:`).
///
/// Returns `true` when the line was consumed by the header phase — including
/// unrecognized lines, which still push a diagnostic — and `false` once
/// `start:` was already seen, so the caller can hand the line to the body
/// dispatch. When the line is `start:`, it is registered as the `start`
/// label on the current line index.
pub fn process_header_line<'b>(
    state: &mut super::body::CompileState<'b>,
    body: &'b str,
    trimmed: &str,
    line_number: usize,
    file: &str,
) -> bool {
    if !state.found_start {
        if body == "start:" {
            state.found_start = true;
            state.labels.insert("start".to_string(), state.lines.len());
            state.lines.push(Line::Label {
                name: "start".to_string(),
                line: line_number,
            });
            return true;
        }

        // A `name:` before `start:` is not a valid label.
        if body.ends_with(':') {
            state.diagnostics.push(Diagnostic::new(
                file,
                line_number,
                format!("unrecognized line: '{trimmed}'"),
            ));
            return true;
        }

        // Header key-value line: `key value`.
        if let Some((key, value)) = split_header(body) {
            if state.seen_keys.contains(&key) {
                state.diagnostics.push(Diagnostic::new(
                    file,
                    line_number,
                    format!("repeated header key '{key}'"),
                ));
            } else {
                state.seen_keys.push(key);
            }

            match key {
                "creature" => {
                    state.species_name = Some(value.trim().to_string());
                }
                "faction" => {
                    state.faction = Some(value.trim().to_string());
                }
                "health" => {
                    let parsed = value.trim().parse::<i64>();
                    if let Ok(n) = parsed {
                        state.health = Some(n);
                    }
                    validate_header_value(
                        key,
                        value,
                        line_number,
                        file,
                        parsed,
                        &mut state.diagnostics,
                    );
                }
                "vision" => {
                    let parsed = value.trim().parse::<i64>();
                    if let Ok(n) = parsed {
                        state.vision = Some(n);
                    }
                    validate_header_value(
                        key,
                        value,
                        line_number,
                        file,
                        parsed,
                        &mut state.diagnostics,
                    );
                }
                "lifespan" => {
                    let parsed = value.trim().parse::<i64>();
                    if let Ok(n) = parsed {
                        state.lifespan = Some(n);
                    }
                    validate_header_value(
                        key,
                        value,
                        line_number,
                        file,
                        parsed,
                        &mut state.diagnostics,
                    );
                }
                _ => {
                    state.diagnostics.push(Diagnostic::new(
                        file,
                        line_number,
                        format!("unknown header key '{key}'"),
                    ));
                }
            }

            return true;
        }

        state.diagnostics.push(Diagnostic::new(
            file,
            line_number,
            format!("unrecognized line: '{trimmed}'"),
        ));
        return true;
    }

    false
}

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
