//! Body-phase dispatch by line shape (label, assignment, goto, if,
//! action call, unrecognized). Extracted from `compile/body.rs`.

use super::action::{parse_action_call, ActionOutcome};
use super::assignment::{looks_like_assignment, parse_assignment};
use crate::actions::Catalog;
use crate::diagnostic::Diagnostic;
use crate::expr;
use crate::perceptions::PerceptionCatalog;
use crate::program::Line;

/// Dispatches one body line (after `start:`) by its shape.
/// Returns `true` when the line was consumed (move to the next line).
pub fn dispatch_body_line(
    state: &mut super::body::CompileState,
    body: &str,
    trimmed: &str,
    line_number: usize,
    file: &str,
    catalog: &Catalog,
    perception_catalog: &PerceptionCatalog,
) -> bool {
    // Label definition: `name:`.
    if body.ends_with(':') {
        let name = body.trim_end_matches(':').trim();
        if state.labels.contains_key(name) {
            state.diagnostics.push(Diagnostic::new(
                file,
                line_number,
                format!("duplicate label '{name}'"),
            ));
            return true;
        }
        state.labels.insert(name.to_string(), state.lines.len());
        state.lines.push(Line::Label {
            name: name.to_string(),
            line: line_number,
        });
        return true;
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
            &mut state.diagnostics,
        ) {
            state.lines.push(line);
        }
        return true;
    }

    // goto: `goto name`.
    if let Some(rest) = body.strip_prefix("goto") {
        let rest = rest.trim();
        if rest.is_empty() {
            state.diagnostics.push(Diagnostic::new(
                file,
                line_number,
                format!("unrecognized line: '{trimmed}'"),
            ));
            return true;
        }
        state.label_refs.push(super::body::LabelRef {
            label: rest.to_string(),
            line: line_number,
        });
        state.lines.push(Line::Goto {
            label: rest.to_string(),
            line: line_number,
        });
        return true;
    }

    // if: `if expr goto name`.
    if let Some(rest) = body.strip_prefix("if") {
        let rest = rest.trim();
        let goto_idx = match rest.rfind(" goto ") {
            Some(i) => i,
            None => {
                state.diagnostics.push(Diagnostic::new(
                    file,
                    line_number,
                    format!("unrecognized line: '{trimmed}'"),
                ));
                return true;
            }
        };
        let expr_text = rest[..goto_idx].trim();
        let label = rest[goto_idx + 6..].trim();
        if expr_text.is_empty() || label.is_empty() {
            state.diagnostics.push(Diagnostic::new(
                file,
                line_number,
                format!("unrecognized line: '{trimmed}'"),
            ));
            return true;
        }
        state.label_refs.push(super::body::LabelRef {
            label: label.to_string(),
            line: line_number,
        });
        match expr::parse(expr_text, line_number, file, catalog.functions()) {
            Ok(expr) => {
                state.lines.push(Line::IfGoto {
                    expr,
                    label: label.to_string(),
                    line: line_number,
                });
            }
            Err(d) => {
                state.diagnostics.push(d);
            }
        }
        return true;
    }

    // Action call: `name(arg, ...)`.
    match parse_action_call(body, catalog, line_number, file, &mut state.diagnostics) {
        ActionOutcome::Resolved(line) => {
            state.lines.push(line);
            return true;
        }
        ActionOutcome::Failed => return true, // diagnostic already pushed
        ActionOutcome::NotAnAction => {}
    }

    false
}
