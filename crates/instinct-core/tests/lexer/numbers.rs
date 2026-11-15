//! Number lexing and out-of-range integer errors.

use instinct_core::lexer::{lex, SpannedToken, Token};

#[test]
fn test_lex_number() {
    let tokens = lex("42", 1, "test.ins").unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Number(42),
            line: 1,
        }]
    );
}

#[test]
fn test_lex_integer_out_of_range() {
    let err = lex("999999999999999999999", 1, "test.ins").unwrap_err();
    assert_eq!(err.line, 1);
    assert_eq!(err.message, "integer literal out of range");
}
