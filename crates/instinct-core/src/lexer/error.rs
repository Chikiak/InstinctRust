//! Lexing errors and their conversion into diagnostics.

use crate::diagnostic::Diagnostic;

/// A lexing error with the file and line where it occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub file: String,
    pub message: String,
    pub line: usize,
}

impl LexError {
    pub fn new(file: impl Into<String>, message: impl Into<String>, line: usize) -> Self {
        LexError {
            file: file.into(),
            message: message.into(),
            line,
        }
    }
}

impl From<LexError> for Diagnostic {
    fn from(err: LexError) -> Self {
        Diagnostic::new(err.file, err.line, err.message)
    }
}
