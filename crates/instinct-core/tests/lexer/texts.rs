//! Text (string) literals: empty text and `#` inside a text.

use instinct_core::lexer::{lex, SpannedToken, Token};

#[test]
fn test_lex_empty_text() {
    let tokens = lex(r#""""#, 1, "test.ins").unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Text("".to_string()),
            line: 1,
        }]
    );
}

#[test]
fn test_lex_text_with_hash() {
    let tokens = lex(r#""a#b""#, 1, "test.ins").unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Text("a#b".to_string()),
            line: 1,
        }]
    );
}

#[test]
fn test_lexer_hash_inside_string_is_not_a_comment() {
    let tokens = lex(r#""a#b""#, 1, "test.ins").unwrap();
    let kinds: Vec<&Token> = tokens.iter().map(|t| &t.kind).collect();
    assert_eq!(kinds, vec![&Token::Text("a#b".to_string())]);
}
