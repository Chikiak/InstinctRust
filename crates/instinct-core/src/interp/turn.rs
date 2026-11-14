//! Execution of one full creature turn, including the line limit.

use crate::diagnostic::Diagnostic;
use crate::expr::EvalContext;
use crate::interp::lines::execute_line;
use crate::interp::state::{CreatureTurnState, TurnResult};
use crate::program::Program;
use crate::world::CreatureState;

/// Maximum number of lines executed per turn (labels count).
///
/// The limit is checked **before** a line is counted as executed, so a turn
/// executes at most `LINE_LIMIT` lines and `pc` is always left pointing at a
/// valid line: the first non-executed one (or the restart point at the end
/// of the program).
const LINE_LIMIT: usize = 100;

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
    match &state.state {
        CreatureState::Sleeping(n) => {
            if *n > 0 {
                state.state = if *n == 1 {
                    CreatureState::Awake
                } else {
                    CreatureState::Sleeping(n - 1)
                };
                return Ok(TurnResult {
                    pending: None,
                    state: state.clone(),
                });
            } else {
                state.state = CreatureState::Awake;
            }
        }
        CreatureState::Awake => {}
    }

    let mut lines_executed = 0usize;

    while state.pc < program.lines.len() {
        if lines_executed >= LINE_LIMIT {
            // Line limit reached: exactly LINE_LIMIT lines were executed and
            // pc points at the first non-executed line, which is valid
            // because the loop only runs while pc < len.
            return Ok(TurnResult {
                pending: None,
                state: state.clone(),
            });
        }

        let line = &program.lines[state.pc];
        lines_executed += 1;

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
            state: state.state.clone(),
            variables: state.variables.clone(),
        },
    })
}
