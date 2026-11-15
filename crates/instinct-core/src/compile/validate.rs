//! Post-compilation validation extracted from `compile/body.rs`.
//!
//! Runs after the body phase: missing required header keys (reported on line
//! 1), `Header` assembly when all five values are present, unknown-label
//! checks for `goto`/`if goto`, the required `start:` label, and final
//! diagnostic ordering by source line. Diagnostics accumulate; none of these
//! checks abort the rest of the file.

use crate::diagnostic::Diagnostic;

/// Runs the post-compilation validation passes over `state` and orders the
/// diagnostics by source line. Extracted verbatim from `compile()`'s
/// post-validation block in `compile/body.rs`.
pub fn finalize_validation(state: &mut super::body::CompileState<'_>, file: &str) {
    // Missing required header keys are reported on line 1.
    if state.found_start {
        for key in super::header::HEADER_KEYS {
            if !state.seen_keys.contains(key) {
                state.diagnostics.push(Diagnostic::new(
                    file,
                    1,
                    format!("missing required header key '{key}'"),
                ));
            }
        }
    }

    // Build the Header only when all five values were collected.
    if let (Some(species_name), Some(faction), Some(health), Some(vision), Some(lifespan)) = (
        state.species_name.take(),
        state.faction.take(),
        state.health.take(),
        state.vision.take(),
        state.lifespan.take(),
    ) {
        state.header = Some(super::header::Header {
            species_name,
            faction,
            health,
            vision,
            lifespan,
        });
    }

    // `goto` / `if goto` to a missing label: report on the goto line.
    for r in &state.label_refs {
        if r.label != "start" && !state.labels.contains_key(&r.label) {
            state.diagnostics.push(Diagnostic::new(
                file,
                r.line,
                format!("goto to unknown label '{}'", r.label),
            ));
        }
    }

    if !state.found_start {
        state
            .diagnostics
            .push(Diagnostic::new(file, 1, "missing required label 'start'"));
    }

    // Report all diagnostics ordered by source line.
    state.diagnostics.sort_by_key(|d| d.line);
}
