//! Assignment parsing: `name = expr` shape detection and semantic checks.

use crate::actions::Catalog;
use crate::diagnostic::Diagnostic;
use crate::expr;
use crate::perceptions::PerceptionCatalog;
use crate::program::Line;

/// Reserved words that may not be used as variable names.
pub const RESERVED_WORDS: &[&str] = &["if", "goto", "and", "or", "not"];

/// Returns `true` when `body` has the shape of an assignment (`name = ...`)
/// with a valid name and a non-`==` equals sign.
///
/// A valid name starts with a letter or `_` and continues with letters,
/// digits or `_`. The equals sign must be a single `=`: `==` on either side
/// disqualifies the line as an assignment.
pub fn looks_like_assignment(body: &str) -> bool {
    let eq_idx = match body.find('=') {
        Some(i) => i,
        None => return false,
    };
    let name = body[..eq_idx].trim();
    if name.is_empty() {
        return false;
    }
    // The name must start with a letter or `_`.
    let mut chars = name.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    // Avoid treating `==` as an assignment: neither the character before nor
    // the character after the `=` may itself be `=`.
    let bytes = body.as_bytes();
    !((eq_idx > 0 && bytes[eq_idx - 1] == b'=')
        || (eq_idx + 1 < bytes.len() && bytes[eq_idx + 1] == b'='))
}

/// Parses an assignment `name = expr`. Pushes a diagnostic naming the element
/// when the target is a reserved word, a perception, or an action name.
/// Returns `None` when the line is not an assignment or when a semantic error
/// occurred (the diagnostic has already been pushed).
pub fn parse_assignment(
    body: &str,
    catalog: &Catalog,
    perception_catalog: &PerceptionCatalog,
    line: usize,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<Line> {
    let eq_idx = body.find('=')?;
    let name = body[..eq_idx].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    // Avoid treating `==` as an assignment.
    if eq_idx > 0 && body.as_bytes()[eq_idx - 1] == b'=' {
        return None;
    }
    // Reserved words cannot be assignment targets.
    if RESERVED_WORDS.contains(&name) {
        diagnostics.push(Diagnostic::new(
            file,
            line,
            format!("cannot assign to reserved word '{name}'"),
        ));
        return None;
    }
    // Perception names cannot be assignment targets.
    if perception_catalog.contains(name) {
        diagnostics.push(Diagnostic::new(
            file,
            line,
            format!("cannot assign to perception '{name}'"),
        ));
        return None;
    }
    // Action names cannot be assignment targets.
    if catalog.is_action_name(name) {
        diagnostics.push(Diagnostic::new(
            file,
            line,
            format!("cannot assign to action '{name}'"),
        ));
        return None;
    }
    // Function names cannot be assignment targets.
    if catalog.is_function_name(name) {
        diagnostics.push(Diagnostic::new(
            file,
            line,
            format!("cannot assign to function '{name}'"),
        ));
        return None;
    }
    // Constant names cannot be assignment targets.
    if catalog.is_constant_name(name) {
        diagnostics.push(Diagnostic::new(
            file,
            line,
            format!("cannot assign to constant '{name}'"),
        ));
        return None;
    }
    let expr_text = body[eq_idx + 1..].trim();
    if expr_text.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            line,
            format!("unrecognized line: '{body}'"),
        ));
        return None;
    }
    match expr::parse(expr_text, line, file, catalog.functions()) {
        Ok(expr) => Some(Line::Assign {
            name: name.to_string(),
            expr,
            line,
        }),
        Err(d) => {
            diagnostics.push(d);
            None
        }
    }
}
