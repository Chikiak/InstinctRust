//! Execution of a single program line and goto-target resolution.

use crate::diagnostic::Diagnostic;
use crate::expr::EvalContext;
use crate::interp::state::{CreatureTurnState, PendingAction};
use crate::interp::visitor::InterpreterVisitor;
use crate::program::{Line, Program};
use std::rc::Rc;

/// Resolves the target of a `goto` / `if goto` and checks that it points
/// inside `program.lines`. An unknown label or an out-of-range destination
/// (only reachable when the program is mutated after compilation) is a
/// runtime error naming the label, never an out-of-range index.
pub fn jump_target(
    program: &Program,
    label: &str,
    line: usize,
) -> Result<usize, (Diagnostic, usize)> {
    let target = program.resolve_label(label).ok_or_else(|| {
        (
            Diagnostic::new("interp", line, format!("goto to unknown label '{label}'")),
            line,
        )
    })?;
    if target >= program.lines.len() {
        return Err((
            Diagnostic::new(
                "interp",
                line,
                format!("goto to label '{label}' destination out of range"),
            ),
            line,
        ));
    }
    Ok(target)
}

/// Evaluates a single program line. Returns `Ok(Some(action))` when a
/// turn-ending action is reached, `Ok(None)` when execution should continue
/// to the next line, or `Err` on a runtime error.
pub fn execute_line(
    program: &Program,
    line: &Line,
    ctx: &dyn EvalContext,
    state: &mut CreatureTurnState,
) -> Result<Option<PendingAction>, (Diagnostic, usize)> {
    match line {
        Line::Label { .. } => {
            state.pc += 1;
            Ok(None)
        }
        Line::Goto { label, line } => {
            state.pc = jump_target(program, label, *line)?;
            Ok(None)
        }
        Line::IfGoto { expr, label, line } => {
            let value = expr
                .accept(&mut InterpreterVisitor { ctx })
                .map_err(|msg| (Diagnostic::new("interp", *line, msg), *line))?;
            if value.is_truthy() {
                state.pc = jump_target(program, label, *line)?;
            } else {
                state.pc += 1;
            }
            Ok(None)
        }
        Line::Assign { name, expr, line } => {
            let value = expr
                .accept(&mut InterpreterVisitor { ctx })
                .map_err(|msg| (Diagnostic::new("interp", *line, msg), *line))?;
            state.variables.borrow_mut().insert(name.clone(), value);
            state.pc += 1;
            Ok(None)
        }
        Line::Action { action, args, line } => {
            let mut evaluated = Vec::with_capacity(args.len());
            for arg in args {
                evaluated.push(
                    arg.accept(&mut InterpreterVisitor { ctx })
                        .map_err(|msg| (Diagnostic::new("interp", *line, msg), *line))?,
                );
            }
            state.pc += 1;
            // Both say and non-say actions are returned as pending; the
            // simulation decides whether the turn continues (say does not
            // consume a turn).
            Ok(Some(PendingAction {
                action: Rc::clone(action),
                args: evaluated,
                line: *line,
            }))
        }
    }
}
