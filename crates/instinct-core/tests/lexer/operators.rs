//! Simple and two-character operators.

use instinct_core::lexer::{lex, Token};

use super::helpers::*;

#[test]
fn test_lex_operators() {
    let tokens = lex("+ - * / %", 1, "test.ins").unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(
        kinds,
        vec![
            &Token::Plus,
            &Token::Minus,
            &Token::Star,
            &Token::Slash,
            &Token::Percent,
        ]
    );
}

#[test]
fn test_lex_two_char_operators() {
    let tokens = lex("== != <= >=", 1, "test.ins").unwrap();
    let kinds = kinds_of(&tokens);
    assert_eq!(kinds, vec![&Token::Eq, &Token::Ne, &Token::Le, &Token::Ge]);
}
