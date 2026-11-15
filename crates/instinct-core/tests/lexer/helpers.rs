//! Shared helpers for lexer tests.

use instinct_core::lexer::{SpannedToken, Token};

/// Collects the line number of each token.
pub fn lines_of(tokens: &[SpannedToken]) -> Vec<usize> {
    tokens.iter().map(|t| t.line).collect()
}

/// Collects the kind of each token.
pub fn kinds_of(tokens: &[SpannedToken]) -> Vec<&Token> {
    tokens.iter().map(|t| &t.kind).collect()
}
