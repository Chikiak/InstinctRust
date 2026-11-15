//! Body compilation: the main `compile` entry point.
//!
//! `compile()` orchestrates three phases over a [`CompileState`]: the header
//! phase ([`process_header_line`]), the body-phase dispatch
//! ([`dispatch_body_line`]) and the post-compilation validation
//! ([`finalize_validation`]), then turns the state into a [`CompileResult`].
//! Diagnostics are accumulated and never abort the rest of the file.

use super::comment::strip_trailing_comment;
use super::dispatch::dispatch_body_line;
use super::header::{process_header_line, Header};
use super::validate::finalize_validation;

use crate::actions::Catalog;
use crate::diagnostic::Diagnostic;
use crate::perceptions::PerceptionCatalog;
use crate::program::{Line, Program};
use std::collections::BTreeMap;

/// A reference to a label from a goto/if-goto, collected for later validation.
pub struct LabelRef {
    pub label: String,
    pub line: usize,
}

/// Aggregates all state carried between the header, dispatch and validation
/// phases of `compile()`.
pub struct CompileState<'a> {
    /// Accumulated diagnostics (never abort the rest of the file).
    pub diagnostics: Vec<Diagnostic>,
    /// Compiled body lines, in source order.
    pub lines: Vec<Line>,
    /// Label name -> line index in `lines`.
    pub labels: BTreeMap<String, usize>,
    /// Label references from goto/if-goto, validated after the body phase.
    pub label_refs: Vec<LabelRef>,
    /// Whether `start:` has been seen.
    pub found_start: bool,
    /// Header keys already seen (to detect repetitions).
    pub seen_keys: Vec<&'a str>,
    /// Collected header values (the `Header` is built at the end).
    pub species_name: Option<String>,
    pub faction: Option<String>,
    pub health: Option<i64>,
    pub vision: Option<i64>,
    pub lifespan: Option<i64>,
    /// Built `Header`, only when all five values were collected.
    pub header: Option<Header>,
}

impl<'a> CompileState<'a> {
    fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            lines: Vec::new(),
            labels: BTreeMap::new(),
            label_refs: Vec::new(),
            found_start: false,
            seen_keys: Vec::new(),
            species_name: None,
            faction: None,
            health: None,
            vision: None,
            lifespan: None,
            header: None,
        }
    }
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
    let mut state = CompileState::new();

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
                state.diagnostics.push(Diagnostic::new(
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

        // Header phase: before `start:`.
        if process_header_line(&mut state, body, trimmed, line_number, file) {
            continue;
        }

        // Body phase: after `start:`.
        if !dispatch_body_line(
            &mut state,
            body,
            trimmed,
            line_number,
            file,
            catalog,
            perception_catalog,
        ) {
            state.diagnostics.push(Diagnostic::new(
                file,
                line_number,
                format!("unrecognized line: '{trimmed}'"),
            ));
        }
    }

    // Post-compilation validation (header keys, Header assembly,
    // unknown-label goto checks, required `start` label).
    finalize_validation(&mut state, file);

    crate::compile::CompileResult {
        program: Program {
            lines: state.lines,
            labels: state.labels,
        },
        diagnostics: state.diagnostics,
        header: state.header,
    }
}
