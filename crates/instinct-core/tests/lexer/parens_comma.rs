//! Parentheses and comma tokens.

use instinct_core::lexer::{lex, Token};

use super::helpers::*;

#[test]
fn test_lex_parens_and_comma() {
    let tokens = lex("(,)", 1, "test.ins").unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(kinds, vec![&Token::LParen, &Token::Comma, &Token::RParen]);
}
