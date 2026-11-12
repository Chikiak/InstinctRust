//! Execution of a single creature turn.
//!
//! The interpreter reads the world immutably (through `CreatureContext`) and
//! evaluates the compiled `Program` starting from the creature's saved pc.
//! It returns at most one pending action (`Option<PendingAction>`); the world
//! applies it afterwards. `say` is returned as a pending action too, and the
//! simulation applies it in the act without ending the turn.

use crate::actions::Action;
use crate::diagnostic::Diagnostic;
use crate::expr::{Binary, Call, EvalContext, ExprVisitor, LiteralInt, LiteralText, Name, Unary};
use crate::program::{Line, Program};
use crate::value::Value;
use crate::world::CreatureState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Maximum number of lines executed per turn (labels count).
///
/// The limit is checked **before** a line is counted as executed, so a turn
/// executes at most `LINE_LIMIT` lines and `pc` is always left pointing at a
/// valid line: the first non-executed one (or the restart point at the end
/// of the program).
const LINE_LIMIT: usize = 100;

/// The state of a creature's turn, kept between turns by the simulation.
#[derive(Debug, Clone)]
pub struct CreatureTurnState {
    /// Next line index to execute.
    pub pc: usize,
    /// Turn cycle state: Awake or Sleeping(n). Replaces `sleep_turns`.
    pub state: CreatureState,
    /// Variables owned by this creature. Shared via `Rc<RefCell>` so the
    /// evaluation context can see assignments made during the same turn.
    pub variables: Rc<RefCell<HashMap<String, Value>>>,
}

impl CreatureTurnState {
    pub fn new() -> Self {
        CreatureTurnState {
            pc: 0,
            state: CreatureState::Awake,
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

/// Resolves the target of a `goto` / `if goto` and checks that it points
/// inside `program.lines`. An unknown label or an out-of-range destination
/// (only reachable when the program is mutated after compilation) is a
/// runtime error naming the label, never an out-of-range index.
fn jump_target(program: &Program, label: &str, line: usize) -> Result<usize, (Diagnostic, usize)> {
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

/// Visitor que evalúa una expresión contra un contexto. Sustituye a la
/// evaluación por nodos: el recorrido pasa a ser el del visitor.
pub struct InterpreterVisitor<'a> {
    pub ctx: &'a dyn crate::expr::EvalContext,
}

impl ExprVisitor for InterpreterVisitor<'_> {
    type Output = Result<Value, String>;

    fn visit_literal_int(&mut self, node: &LiteralInt) -> Self::Output {
        Ok(Value::Int(node.value))
    }

    fn visit_literal_text(&mut self, node: &LiteralText) -> Self::Output {
        Ok(Value::Text(node.value.clone()))
    }

    fn visit_name(&mut self, node: &Name) -> Self::Output {
        self.ctx
            .resolve(&node.name)
            .ok_or_else(|| format!("unknown variable '{}'", node.name))
    }

    fn visit_call(&mut self, node: &Call) -> Self::Output {
        let view = self
            .ctx
            .view()
            .ok_or_else(|| format!("function '{}' has no creature view", node.function.name()))?;
        let mut evaluated = Vec::with_capacity(node.args.len());
        for arg in &node.args {
            evaluated.push(arg.accept(self)?);
        }
        node.function.call(view, &evaluated)
    }

    fn visit_unary(&mut self, node: &Unary) -> Self::Output {
        let value = node.operand.accept(self)?;
        value.unary_op(&node.op)
    }

    fn visit_binary(&mut self, node: &Binary) -> Self::Output {
        let left = node.left.accept(self)?;
        // Short-circuit: only evaluate the right operand when needed.
        match node.op.as_str() {
            "and" => {
                if !left.is_truthy() {
                    return Ok(Value::Int(0));
                }
            }
            "or" => {
                if left.is_truthy() {
                    return Ok(Value::Int(1));
                }
            }
            _ => {}
        }
        let right = node.right.accept(self)?;
        match node.op.as_str() {
            "and" => Ok(Value::Int(if right.is_truthy() { 1 } else { 0 })),
            "or" => Ok(Value::Int(if right.is_truthy() { 1 } else { 0 })),
            _ => left.binary_op(&node.op, &right),
        }
    }
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
