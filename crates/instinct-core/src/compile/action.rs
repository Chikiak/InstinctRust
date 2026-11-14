//! Action call parsing: `name(arg, ...)` resolution against the catalog.

use crate::actions::Catalog;
use crate::diagnostic::Diagnostic;
use crate::expr;
use crate::program::Line;

/// Outcome of parsing an action call line.
#[derive(Debug)]
pub enum ActionOutcome {
    /// The line is not an action call; compilation keeps looking at other
    /// line forms (and may end up as "unrecognized").
    NotAnAction,
    /// The line resolved to a valid action line.
    Resolved(Line),
    /// The line was an action call but a diagnostic was already pushed
    /// (bad argument, unknown action, or wrong arity); nothing more to do.
    Failed,
}

/// Divides `args_text` into top-level comma-separated arguments.
/// Commas and parentheses inside a quoted text (`"..."`) or inside nested
/// parentheses do not break an argument, so `say(see(1, 0))` is a single
/// argument and its inner comma is not split.
pub fn split_action_args(args_text: &str) -> Vec<&str> {
    let mut parts: Vec<&str> = Vec::new();
    let mut depth = 0usize;
    let mut in_text = false;
    let mut start = 0;
    let mut chars = args_text.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if in_text {
            if ch == '\\' {
                let _ = chars.next(); // escape: skip the following character
            } else if ch == '"' {
                in_text = false;
            }
            continue;
        }
        match ch {
            '"' => in_text = true,
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&args_text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&args_text[start..]);
    parts
}

/// Parses an action call `name(arg, ...)`. Pushes a diagnostic naming the
/// action when it is unknown or has the wrong arity, or naming the argument
/// when it fails to parse. The diagnostic is never silently dropped.
pub fn parse_action_call(
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
        for a in split_action_args(args_text)
            .into_iter()
            .map(|a| a.trim())
            .filter(|a| !a.is_empty())
        {
            match expr::parse(a, line, file, catalog.functions()) {
                Ok(expr) => args.push(expr),
                Err(d) => {
                    // Propagate the argument parse error as a diagnostic
                    // (with its line and file) instead of silently dropping it.
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
