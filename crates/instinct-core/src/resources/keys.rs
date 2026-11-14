//! Key-value line parsing for `.te` and `.ob` resource files.
//! All helpers are private to the `resources` module tree.

use crate::diagnostic::Diagnostic;

/// Result of loading a single resource: the parsed value plus diagnostics.
pub type LoadResult<T> = (Option<T>, Vec<Diagnostic>);

/// Splits a line into (key, value) using the first whitespace as separator.
/// Returns None for blank lines, comments, and lines without a key.
pub(super) fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    // Comment line: its first non-blank character is '#'.
    if line.starts_with('#') {
        return None;
    }
    match line.find(char::is_whitespace) {
        Some(i) => {
            let key = &line[..i];
            let value = line[i..].trim();
            if key.is_empty() {
                None
            } else {
                Some((key, value))
            }
        }
        None => Some((line, "")),
    }
}

/// Reads all key-value pairs from a content, in order of appearance,
/// ignoring blanks and comments. Returns (line number, key, value) triples
/// where the line number is counted from 1 for diagnostics.
pub(super) fn read_keys(content: &str) -> Vec<(usize, String, String)> {
    let mut output: Vec<(usize, String, String)> = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let number = idx + 1;
        if let Some((key, value)) = split_key_value(line) {
            output.push((number, key.to_string(), value.to_string()));
        }
    }
    output
}

/// Value of a key looked up by name; returns the line number of its first
/// appearance or None if it does not exist.
pub(super) fn key_value<'a>(
    keys: &'a [(usize, String, String)],
    name: &'a str,
) -> Option<(usize, &'a str)> {
    keys.iter()
        .find(|(_, k, _)| k == name)
        .map(|(n, _, v)| (*n, v.as_str()))
}

/// Validates that `value` has exactly one character.
pub(super) fn validate_single_character(
    number: usize,
    value: &str,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<char> {
    let mut chars = value.chars();
    let first = chars.next();
    let second = chars.next();
    match (first, second) {
        (Some(c), None) => Some(c),
        _ => {
            diagnostics.push(Diagnostic::new(
                file,
                number,
                format!("the value of 'char' must be a single character, got: '{value}'"),
            ));
            None
        }
    }
}

/// Parses an i64 integer; fails with a diagnostic if not numeric.
pub(super) fn parse_integer(
    number: usize,
    value: &str,
    key: &str,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match value.trim().parse::<i64>() {
        Ok(v) => Some(v),
        Err(_) => {
            diagnostics.push(Diagnostic::new(
                file,
                number,
                format!("the value of '{key}' must be an integer, got: '{value}'"),
            ));
            None
        }
    }
}
