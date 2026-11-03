//! Diagnostic with file, line, and message.
//! Text format: `file:line: message`.

/// A diagnostic produced by compilation, loading, or execution.
/// The line is always counted from 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub message: String,
}

impl Diagnostic {
    /// Builds a new diagnostic.
    pub fn new(file: impl Into<String>, line: usize, message: impl Into<String>) -> Self {
        Diagnostic {
            file: file.into(),
            line,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.message)
    }
}
