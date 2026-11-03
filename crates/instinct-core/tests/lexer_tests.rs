//! Unit tests for the `lexer` module.

use instinct_core::lexer::{lex, Token};

#[test]
fn test_lexer_hash_inside_string_is_not_a_comment() {
    let tokens = lex(r#""a#b""#, 1).unwrap();
    let kinds: Vec<&Token> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(kinds, vec![&Token::Text("a#b".to_string())]);
}

#[test]
fn test_lexer_trailing_comment_ignored() {
    let tokens = lex("x + 1   # this is a comment", 3).unwrap();
    let last_line = tokens.last().map(|t| t.line).unwrap_or(0);
    assert_eq!(last_line, 3);
    assert!(!tokens.iter().any(|t| matches!(t.kind, Token::Text(_))));
}

#[test]
fn test_lexer_unknown_character_reports_line() {
    let err = lex("x @ 1", 5).unwrap_err();
    assert_eq!(err.line, 5);
    assert!(err.message.contains('@'));
}

#[test]
fn test_lexer_unterminated_string_reports_line() {
    let err = lex("unterminated \"", 2).unwrap_err();
    assert_eq!(err.line, 2);
}
