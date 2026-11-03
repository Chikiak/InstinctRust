//! Compiled program: a flat list of lines with source line numbers and
//! resolved labels. Each line is one statement of the Instinct body.

use crate::actions::Action;
use crate::expr::Expr;
use std::collections::HashMap;
use std::rc::Rc;

/// A compiled line: either a label definition, a goto, a conditional goto,
/// an assignment, or an action call.
#[derive(Debug)]
pub enum Line {
    /// A label definition: `name:`. Execution falls through to the next line.
    Label { name: String, line: usize },
    /// Unconditional jump: `goto name`.
    Goto { label: String, line: usize },
    /// Conditional jump: `if expr goto name`.
    IfGoto {
        expr: Box<dyn Expr>,
        label: String,
        line: usize,
    },
    /// Assignment: `name = expr`.
    Assign {
        name: String,
        expr: Box<dyn Expr>,
        line: usize,
    },
    /// Action call: `action(arg, ...)`.
    Action {
        action: Rc<dyn Action>,
        args: Vec<Box<dyn Expr>>,
        line: usize,
    },
}

impl Clone for Line {
    fn clone(&self) -> Self {
        match self {
            Line::Label { name, line } => Line::Label {
                name: name.clone(),
                line: *line,
            },
            Line::Goto { label, line } => Line::Goto {
                label: label.clone(),
                line: *line,
            },
            Line::IfGoto { expr, label, line } => Line::IfGoto {
                expr: expr.clone_box(),
                label: label.clone(),
                line: *line,
            },
            Line::Assign { name, expr, line } => Line::Assign {
                name: name.clone(),
                expr: expr.clone_box(),
                line: *line,
            },
            Line::Action { action, args, line } => Line::Action {
                action: Rc::clone(action),
                args: args.iter().map(|a| a.clone_box()).collect(),
                line: *line,
            },
        }
    }
}

/// A compiled program: flat lines (each keeping its source line number) and a
/// label table mapping names to line indices.
#[derive(Debug, Clone)]
pub struct Program {
    pub lines: Vec<Line>,
    /// Label name -> index into `lines`.
    pub labels: HashMap<String, usize>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            lines: Vec::new(),
            labels: HashMap::new(),
        }
    }

    /// Resolves a label name to the index of its line. Returns `None` when the
    /// label does not exist.
    pub fn resolve_label(&self, name: &str) -> Option<usize> {
        self.labels.get(name).copied()
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
