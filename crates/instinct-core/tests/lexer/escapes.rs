//! Escape sequences inside text literals.

use instinct_core::lexer::{lex, SpannedToken, Token};

#[test]
fn test_lex_text_escapes() {
    let tokens = lex(r#""a\"b\\c\d""#, 1, "test.ins").unwrap();
    assert_eq!(
        tokens,
        vec![SpannedToken {
            kind: Token::Text("a\"b\\c\\d".to_string()),
            line: 1,
        }]
    );
}
