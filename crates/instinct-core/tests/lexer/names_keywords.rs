//! Name/identifier lexing and keyword recognition.

use instinct_core::lexer::{lex, SpannedToken, Token};

use super::helpers::*;

#[test]
fn test_lex_name() {
    let tokens = lex("foo_bar", 1, "test.ins").unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Name("foo_bar".to_string()),
            line: 1,
        }]
    );
}

#[test]
fn test_lex_keywords() {
    let tokens = lex("and or not", 1, "test.ins").unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(kinds, vec![&Token::And, &Token::Or, &Token::Not]);
    assert_eq!(lines_of(&tokens), vec![1, 1, 1]);
}
