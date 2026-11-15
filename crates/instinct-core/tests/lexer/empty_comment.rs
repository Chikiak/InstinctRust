//! Empty lines and comments produce zero tokens.

use instinct_core::lexer::{lex, Token};

#[test]
fn test_lex_empty_line_no_tokens() {
    assert!(lex("", 1, "test.ins").unwrap().is_empty());
    assert!(lex("   \t", 4, "test.ins").unwrap().is_empty());
}

#[test]
fn test_lexer_trailing_comment_ignored() {
    let tokens = lex("x + 1   # this is a comment", 3, "test.ins").unwrap();
    let last_line = tokens.last().map(|t| t.line).unwrap_or(0);
    assert_eq!(last_line, 3);
    assert!(!tokens.iter().any(|t| matches!(t.kind, Token::Text(_))));
}
