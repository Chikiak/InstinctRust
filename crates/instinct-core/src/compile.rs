//! Compilation of `.ins` creature files: header validation, body parsing,
//! label table, and name/arity checks against the catalog.
//!
//! Produces a [`CompileResult`] holding a [`Program`] plus accumulated
//! diagnostics. Diagnostics never abort compilation of the rest of the file.

use crate::actions::Catalog;
use crate::diagnostic::Diagnostic;
use crate::expr;
use crate::perception::PerceptionCatalog;
use crate::program::{Line, Program};

/// Parsed header of a `.ins` file: the five required attributes.
#[derive(Debug, Clone)]
pub struct Header {
    pub species_name: String,
    pub faction: String,
    pub health: i64,
    pub vision: i64,
    pub lifespan: i64,
}

/// Result of compiling a `.ins` file: the compiled program plus diagnostics
/// and, when the header is complete and valid, the parsed header attributes.
#[derive(Debug, Clone)]
pub struct CompileResult {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
    pub header: Option<Header>,
}

impl CompileResult {
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// The five required header keys, in no particular order.
const HEADER_KEYS: &[&str] = &["creature", "faction", "health", "vision", "lifespan"];

/// Reserved words that may not be used as variable names.
const RESERVED_WORDS: &[&str] = &["if", "goto", "and", "or", "not"];

/// A reference to a label from a goto/if-goto, collected for later validation.
struct LabelRef {
    label: String,
    line: usize,
}

/// Compiles a source text into a `Program`, validating header, labels, names
/// and arity against the catalogs. Diagnostics are accumulated and never abort
/// the rest of the file.
pub fn compile(
    text: &str,
    catalog: &Catalog,
    perception_catalog: &PerceptionCatalog,
    file: &str,
) -> CompileResult {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut lines: Vec<Line> = Vec::new();
    let mut labels: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut label_refs: Vec<LabelRef> = Vec::new();
    let mut found_start = false;
    let mut seen_keys: Vec<&str> = Vec::new();
    let mut header: Option<Header> = None;

    // Collected header values (parsed as we go; final Header built at the end).
    let mut species_name: Option<String> = None;
    let mut faction: Option<String> = None;
    let mut health: Option<i64> = None;
    let mut vision: Option<i64> = None;
    let mut lifespan: Option<i64> = None;

    for (idx, raw_line) in text.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = raw_line.trim();

        // Blank lines and full-line comments are ignored.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Strip a trailing comment (`#` outside a quoted text).
        let body = match strip_trailing_comment(trimmed) {
            Some(b) => b,
            None => {
                diagnostics.push(Diagnostic::new(
                    file,
                    line_number,
                    format!("unterminated text literal in line: '{trimmed}'"),
                ));
                continue;
            }
        };

        let body = body.trim();
        if body.is_empty() {
            continue;
        }

        // -------------------------------------------------------------------
        // Header phase: before `start:`.
        // -------------------------------------------------------------------
        if !found_start {
            if body == "start:" {
                found_start = true;
                labels.insert("start".to_string(), lines.len());
                lines.push(Line::Label {
                    name: "start".to_string(),
                    line: line_number,
                });
                continue;
            }

            // A `name:` before `start:` is not a valid label.
            if body.ends_with(':') {
                diagnostics.push(Diagnostic::new(
                    file,
                    line_number,
                    format!("unrecognized line: '{trimmed}'"),
                ));
                continue;
            }

            // Header key-value line: `key value`.
            if let Some((key, value)) = split_header(body) {
                if seen_keys.contains(&key) {
                    diagnostics.push(Diagnostic::new(
                        file,
                        line_number,
                        format!("repeated header key '{key}'"),
                    ));
                } else {
                    seen_keys.push(key);
                }

                match key {
                    "creature" => {
                        species_name = Some(value.trim().to_string());
                    }
                    "faction" => {
                        faction = Some(value.trim().to_string());
                    }
                    "health" => {
                        let parsed = value.trim().parse::<i64>();
                        if let Ok(n) = parsed {
                            health = Some(n);
                        }
                        validate_header_value(
                            key,
                            value,
                            line_number,
                            file,
                            parsed,
                            &mut diagnostics,
                        );
                    }
                    "vision" => {
                        let parsed = value.trim().parse::<i64>();
                        if let Ok(n) = parsed {
                            vision = Some(n);
                        }
                        validate_header_value(
                            key,
                            value,
                            line_number,
                            file,
                            parsed,
                            &mut diagnostics,
                        );
                    }
                    "lifespan" => {
                        let parsed = value.trim().parse::<i64>();
                        if let Ok(n) = parsed {
                            lifespan = Some(n);
                        }
                        validate_header_value(
                            key,
                            value,
                            line_number,
                            file,
                            parsed,
                            &mut diagnostics,
                        );
                    }
                    _ => {
                        diagnostics.push(Diagnostic::new(
                            file,
                            line_number,
                            format!("unknown header key '{key}'"),
                        ));
                    }
                }

                continue;
            }

            diagnostics.push(Diagnostic::new(
                file,
                line_number,
                format!("unrecognized line: '{trimmed}'"),
            ));
            continue;
        }

        // -------------------------------------------------------------------
        // Body phase: after `start:`.
        // -------------------------------------------------------------------

        // Label definition: `name:`.
        if body.ends_with(':') {
            let name = body.trim_end_matches(':').trim();
            if labels.contains_key(name) {
                diagnostics.push(Diagnostic::new(
                    file,
                    line_number,
                    format!("duplicate label '{name}'"),
                ));
                continue;
            }
            labels.insert(name.to_string(), lines.len());
            lines.push(Line::Label {
                name: name.to_string(),
                line: line_number,
            });
            continue;
        }

        // Assignment: `name = expr` (checked before `if` so that `if = 5` is
        // treated as an assignment to a reserved word, not an `if` statement).
        if looks_like_assignment(body) {
            if let Some(line) = parse_assignment(
                body,
                catalog,
                perception_catalog,
                line_number,
                file,
                &mut diagnostics,
            ) {
                lines.push(line);
            }
            continue;
        }

        // goto: `goto name`.
        if let Some(rest) = body.strip_prefix("goto") {
            let rest = rest.trim();
            if rest.is_empty() {
                diagnostics.push(Diagnostic::new(
                    file,
                    line_number,
                    format!("unrecognized line: '{trimmed}'"),
                ));
                continue;
            }
            label_refs.push(LabelRef {
                label: rest.to_string(),
                line: line_number,
            });
            lines.push(Line::Goto {
                label: rest.to_string(),
                line: line_number,
            });
            continue;
        }

        // if: `if expr goto name`.
        if let Some(rest) = body.strip_prefix("if") {
            let rest = rest.trim();
            let goto_idx = match rest.rfind(" goto ") {
                Some(i) => i,
                None => {
                    diagnostics.push(Diagnostic::new(
                        file,
                        line_number,
                        format!("unrecognized line: '{trimmed}'"),
                    ));
                    continue;
                }
            };
            let expr_text = rest[..goto_idx].trim();
            let label = rest[goto_idx + 6..].trim();
            if expr_text.is_empty() || label.is_empty() {
                diagnostics.push(Diagnostic::new(
                    file,
                    line_number,
                    format!("unrecognized line: '{trimmed}'"),
                ));
                continue;
            }
            label_refs.push(LabelRef {
                label: label.to_string(),
                line: line_number,
            });
            match expr::parse(expr_text, line_number, file) {
                Ok(expr) => {
                    lines.push(Line::IfGoto {
                        expr,
                        label: label.to_string(),
                        line: line_number,
                    });
                }
                Err(d) => {
                    diagnostics.push(d);
                }
            }
            continue;
        }

        // Assignment: `name = expr`.
        if looks_like_assignment(body) {
            if let Some(line) = parse_assignment(
                body,
                catalog,
                perception_catalog,
                line_number,
                file,
                &mut diagnostics,
            ) {
                lines.push(line);
            }
            // Assignment shape consumed: do not fall through to "unrecognized".
            continue;
        }

        // Action call: `name(arg, ...)`.
        match parse_action_call(body, catalog, line_number, file, &mut diagnostics) {
            ActionOutcome::Resolved(line) => {
                lines.push(line);
                continue;
            }
            ActionOutcome::Failed => continue, // diagnostic already pushed
            ActionOutcome::NotAnAction => {}
        }

        diagnostics.push(Diagnostic::new(
            file,
            line_number,
            format!("unrecognized line: '{trimmed}'"),
        ));
    }

    // -----------------------------------------------------------------------
    // Post-compilation validation.
    // -----------------------------------------------------------------------

    // Missing required header keys are reported on line 1.
    if found_start {
        for key in HEADER_KEYS {
            if !seen_keys.contains(key) {
                diagnostics.push(Diagnostic::new(
                    file,
                    1,
                    format!("missing required header key '{key}'"),
                ));
            }
        }
    }

    // Build the Header only when all five values were collected.
    if let (Some(species_name), Some(faction), Some(health), Some(vision), Some(lifespan)) =
        (species_name, faction, health, vision, lifespan)
    {
        header = Some(Header {
            species_name,
            faction,
            health,
            vision,
            lifespan,
        });
    }

    // `goto` / `if goto` to a missing label: report on the goto line.
    for r in &label_refs {
        if r.label != "start" && !labels.contains_key(&r.label) {
            diagnostics.push(Diagnostic::new(
                file,
                r.line,
                format!("goto to unknown label '{}'", r.label),
            ));
        }
    }

    if !found_start {
        diagnostics.push(Diagnostic::new(file, 1, "missing required label 'start'"));
    }

    // Report all diagnostics ordered by source line.
    diagnostics.sort_by_key(|d| d.line);

    CompileResult {
        program: Program { lines, labels },
        diagnostics,
        header,
    }
}

/// Returns `true` when `body` has the shape of an assignment (`name = ...`)
/// with a valid name and a non-`==` equals sign.
/// Returns `true` when `body` has the shape of an assignment (`name = ...`)
/// with a valid name and a non-`==` equals sign.
///
/// A valid name starts with a letter or `_` and continues with letters,
/// digits or `_`. The equals sign must be a single `=`: `==` on either side
/// disqualifies the line as an assignment.
fn looks_like_assignment(body: &str) -> bool {
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
    if (eq_idx > 0 && bytes[eq_idx - 1] == b'=')
        || (eq_idx + 1 < bytes.len() && bytes[eq_idx + 1] == b'=')
    {
        return false;
    }
    true
}

/// Validates that a header value is an integer > 0 (vision >= 1). Pushes a
/// diagnostic naming the key when invalid.
///
/// The value is parsed exactly once (the caller already parsed it and passes
/// the result); an invalid integer or a value below the minimum produces a
/// single diagnostic on the value's line.
fn validate_header_value(
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

/// Splits a header line `key value` into (key, value). Returns None when the
/// line is not a simple key-value pair.
fn split_header(line: &str) -> Option<(&str, &str)> {
    let idx = line.find(char::is_whitespace)?;
    let key = &line[..idx];
    let value = line[idx..].trim();
    if key.is_empty() || value.is_empty() {
        None
    } else {
        Some((key, value))
    }
}

/// Parses an assignment `name = expr`. Pushes a diagnostic naming the element
/// when the target is a reserved word, a perception, or an action name.
/// Returns `None` when the line is not an assignment or when a semantic error
/// occurred (the diagnostic has already been pushed).
fn parse_assignment(
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
    let expr_text = body[eq_idx + 1..].trim();
    if expr_text.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            line,
            format!("unrecognized line: '{body}'"),
        ));
        return None;
    }
    match expr::parse(expr_text, line, file) {
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

/// Outcome of parsing an action call line.
enum ActionOutcome {
    /// The line is not an action call; compilation keeps looking at other
    /// line forms (and may end up as "unrecognized").
    NotAnAction,
    /// The line resolved to a valid action line.
    Resolved(Line),
    /// The line was an action call but a diagnostic was already pushed
    /// (bad argument, unknown action, or wrong arity); nothing more to do.
    Failed,
}

/// Parses an action call `name(arg, ...)`. Pushes a diagnostic naming the
/// action when it is unknown or has the wrong arity, or naming the argument
/// when it fails to parse. The diagnostic is never silently dropped.
fn parse_action_call(
    body: &str,
    catalog: &Catalog,
    line: usize,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> ActionOutcome {
    let paren = match body.find('(') {
        Some(p) => p,
        None => return ActionOutcome::NotAnAction,
    };
    let name = body[..paren].trim();
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return ActionOutcome::NotAnAction;
    }
    let rest = body[paren + 1..].trim();
    let closing = match rest.rfind(')') {
        Some(c) => c,
        None => return ActionOutcome::NotAnAction,
    };
    let args_text = &rest[..closing];
    let after = rest[closing + 1..].trim();
    if !after.is_empty() {
        return ActionOutcome::NotAnAction;
    }

    let args: Vec<Box<dyn expr::Expr>> = if args_text.trim().is_empty() {
        Vec::new()
    } else {
        let mut args = Vec::new();
        for a in args_text
            .split(',')
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
        {
            match expr::parse(a, line, file) {
                Ok(expr) => args.push(expr),
                Err(d) => {
                    // Propaga el error de parseo del argumento como diagnóstico
                    // (con su línea y archivo) en lugar de descartarlo en silencio.
                    diagnostics.push(d);
                    return ActionOutcome::Failed;
                }
            }
        }
        args
    };

    let arg_count = args.len();
    match catalog.resolve(name, arg_count) {
        Ok(action) => ActionOutcome::Resolved(Line::Action { action, args, line }),
        Err(d) => {
            diagnostics.push(Diagnostic::new(file, line, d.message));
            ActionOutcome::Failed
        }
    }
}

/// Strips a trailing comment (`# ...`) that is outside any quoted text.
/// Returns `None` if a text literal is unterminated, otherwise a slice of
/// `text` up to (but not including) the comment start.
fn strip_trailing_comment(text: &str) -> Option<&str> {
    let mut in_text = false;
    for (i, ch) in text.char_indices() {
        if in_text {
            if ch == '"' {
                in_text = false;
            }
        } else if ch == '"' {
            in_text = true;
        } else if ch == '#' {
            return Some(&text[..i]);
        }
    }
    if in_text {
        None
    } else {
        Some(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::{Catalog, Wait};
    use std::rc::Rc;

    fn catalog_with_wait() -> Catalog {
        let mut c = Catalog::new();
        let _ = c.register(Rc::new(Wait));
        c
    }

    #[test]
    fn test_compile_minimal_start() {
        let cat = catalog_with_wait();
        let pc = PerceptionCatalog::default();
        let result = compile(
            "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\n",
            &cat,
            &pc,
            "test.ins",
        );
        assert!(
            result.diagnostics.is_empty(),
            "no diagnostics, got: {:?}",
            result.diagnostics
        );
        assert!(result.program.resolve_label("start").is_some());
    }

    #[test]
    fn test_compile_duplicate_label() {
        let cat = catalog_with_wait();
        let pc = PerceptionCatalog::default();
        let result = compile(
            "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nfoo:\nfoo:\n",
            &cat,
            &pc,
            "test.ins",
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("duplicate label 'foo'")),
            "should report duplicate label foo, got: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn test_compile_missing_start() {
        let cat = catalog_with_wait();
        let pc = PerceptionCatalog::default();
        let result = compile(
            "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\ngoto start\n",
            &cat,
            &pc,
            "test.ins",
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.message.contains("missing required label 'start'")),
            "should report missing start, got: {:?}",
            result.diagnostics
        );
    }
}
