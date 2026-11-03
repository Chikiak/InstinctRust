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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_display_format() {
        let d = Diagnostic::new("creatures/walker.ins", 42, "label 'end' not defined");
        assert_eq!(
            d.to_string(),
            "creatures/walker.ins:42: label 'end' not defined"
        );
        assert_eq!(d.file, "creatures/walker.ins");
        assert_eq!(d.line, 42);
        assert_eq!(d.message, "label 'end' not defined");
    }
}
