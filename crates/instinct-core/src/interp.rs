//! Execution of a single creature turn.
//!
//! The interpreter reads the world immutably (through `CreatureContext`) and
//! evaluates the compiled `Program` starting from the creature's saved pc.
//! It returns at most one pending action (`Option<PendingAction>`); the world
//! applies it afterwards. `say` is returned as a pending action too, and the
//! simulation applies it in the act without ending the turn.

use crate::actions::Action;
use crate::diagnostic::Diagnostic;
use crate::expr::EvalContext;
use crate::program::{Line, Program};
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Maximum number of lines executed per turn (labels count).
const LINE_LIMIT: usize = 100;

/// The state of a creature's turn, kept between turns by the simulation.
#[derive(Debug, Clone)]
pub struct CreatureTurnState {
    /// Next line index to execute.
    pub pc: usize,
    /// Turns of sleep remaining.
    pub sleep_turns: u32,
    /// Variables owned by this creature. Shared via `Rc<RefCell>` so the
    /// evaluation context can see assignments made during the same turn.
    pub variables: Rc<RefCell<HashMap<String, Value>>>,
}

impl CreatureTurnState {
    pub fn new() -> Self {
        CreatureTurnState {
            pc: 0,
            sleep_turns: 0,
            variables: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl Default for CreatureTurnState {
    fn default() -> Self {
        Self::new()
    }
}

/// The result of executing a turn.
#[derive(Debug, Clone)]
pub struct TurnResult {
    /// The pending action, if the turn ended on an action.
    pub pending: Option<PendingAction>,
    /// The turn state after execution (pc, sleep, variables).
    pub state: CreatureTurnState,
}

/// A pending action waiting to be applied by the world, with its arguments
/// already evaluated.
#[derive(Debug, Clone)]
pub struct PendingAction {
    pub action: Rc<dyn Action>,
    pub args: Vec<Value>,
    /// Source line number of the action (for RuntimeError reporting).
    pub line: usize,
}

/// Evaluates a single program line. Returns `Ok(Some(action))` when a
/// turn-ending action is reached, `Ok(None)` when execution should continue
/// to the next line, or `Err` on a runtime error.
fn execute_line(
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
            let target = program.resolve_label(label).ok_or_else(|| {
                (
                    Diagnostic::new("interp", *line, format!("goto to unknown label '{label}'")),
                    *line,
                )
            })?;
            state.pc = target;
            Ok(None)
        }
        Line::IfGoto { expr, label, line } => {
            let value = expr
                .eval(ctx)
                .map_err(|msg| (Diagnostic::new("interp", *line, msg), *line))?;
            if value.is_truthy() {
                let target = program.resolve_label(label).ok_or_else(|| {
                    (
                        Diagnostic::new(
                            "interp",
                            *line,
                            format!("goto to unknown label '{label}'"),
                        ),
                        *line,
                    )
                })?;
                state.pc = target;
            } else {
                state.pc += 1;
            }
            Ok(None)
        }
        Line::Assign { name, expr, line } => {
            let value = expr
                .eval(ctx)
                .map_err(|msg| (Diagnostic::new("interp", *line, msg), *line))?;
            state.variables.borrow_mut().insert(name.clone(), value);
            state.pc += 1;
            Ok(None)
        }
        Line::Action { action, args, line } => {
            let mut evaluated = Vec::with_capacity(args.len());
            for arg in args {
                evaluated.push(
                    arg.eval(ctx)
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

/// Executes one turn of a creature.
///
/// `program` is the compiled program; `ctx` provides read-only access to
/// perceptions and variables; `state` is the creature's turn state (pc,
/// sleep, variables).
///
/// Returns the turn result with at most one pending action, or a runtime
/// error that the simulation turns into a `RuntimeError` event.
pub fn execute_turn(
    program: &Program,
    ctx: &dyn EvalContext,
    state: &mut CreatureTurnState,
) -> Result<TurnResult, (Diagnostic, usize)> {
    // Sleep: decrement and skip execution this turn. pc is unchanged so the
    // creature resumes at the same instruction when it wakes.
    if state.sleep_turns > 0 {
        state.sleep_turns -= 1;
        return Ok(TurnResult {
            pending: None,
            state: state.clone(),
        });
    }

    let mut lines_executed = 0usize;

    while state.pc < program.lines.len() {
        let line = &program.lines[state.pc];
        lines_executed += 1;

        if lines_executed > LINE_LIMIT {
            // Line limit reached: turn ends like wait(1). pc is left pointing
            // at the current line so the next turn resumes here.
            return Ok(TurnResult {
                pending: None,
                state: state.clone(),
            });
        }

        match execute_line(program, line, ctx, state)? {
            Some(pending) => {
                return Ok(TurnResult {
                    pending: Some(pending),
                    state: state.clone(),
                });
            }
            None => {
                // pc already advanced by execute_line.
            }
        }
    }

    // End of file: turn ends without action; next turn resumes at start.
    let next_pc = match program.resolve_label("start") {
        Some(pc) => pc,
        None => {
            return Err((
                Diagnostic::new("interp", 0, "missing required label 'start'".to_string()),
                0,
            ));
        }
    };
    Ok(TurnResult {
        pending: None,
        state: CreatureTurnState {
            pc: next_pc,
            sleep_turns: state.sleep_turns,
            variables: state.variables.clone(),
        },
    })
}
