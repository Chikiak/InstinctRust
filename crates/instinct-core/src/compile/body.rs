//! Body compilation: the main `compile` entry point, the header phase
//! (before `start:`) and the body-phase dispatch by line shape.

use super::action::{parse_action_call, ActionOutcome};
use super::assignment::{looks_like_assignment, parse_assignment};
use super::comment::strip_trailing_comment;
use super::header::{split_header, validate_header_value, HEADER_KEYS};
use crate::actions::Catalog;
use crate::diagnostic::Diagnostic;
use crate::expr;
use crate::perceptions::PerceptionCatalog;
use crate::program::{Line, Program};

/// A reference to a label from a goto/if-goto, collected for later validation.
pub struct LabelRef {
    pub label: String,
    pub line: usize,
}

/// Compiles a source text into a `Program`, validating header, labels, names
/// and arity against the catalogs. Diagnostics are accumulated and never abort
/// the rest of the file.
pub fn compile(
    text: &str,
    catalog: &Catalog,
    perception_catalog: &PerceptionCatalog,
    file: &str,
) -> crate::compile::CompileResult {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut lines: Vec<Line> = Vec::new();
    let mut labels: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut label_refs: Vec<LabelRef> = Vec::new();
    let mut found_start = false;
    let mut seen_keys: Vec<&str> = Vec::new();
    let mut header: Option<crate::compile::Header> = None;

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
            match expr::parse(expr_text, line_number, file, catalog.functions()) {
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
        header = Some(crate::compile::Header {
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

    crate::compile::CompileResult {
        program: Program { lines, labels },
        diagnostics,
        header,
    }
}
