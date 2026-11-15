//! Line counting: CR, LF and CRLF each count once.

use instinct_core::lexer::{lex, Token};

use super::helpers::*;

#[test]
fn test_lex_crlf_counts_one_line() {
    let tokens = lex("a\r\nb", 1, "test.ins").unwrap();
    assert_eq!(
        kinds_of(&tokens),
        vec![&Token::Name("a".to_string()), &Token::Name("b".to_string())]
    );
    assert_eq!(lines_of(&tokens), vec![1, 2]);
}

#[test]
fn test_lex_lone_cr_counts_one_line() {
    let tokens = lex("a\rb", 1, "test.ins").unwrap();
    assert_eq!(lines_of(&tokens), vec![1, 2]);
}

#[test]
fn test_lex_multiple_lines_numbered_correctly() {
    let tokens = lex("a 1\r\nb 2\nc 3", 1, "test.ins").unwrap();
    assert_eq!(lines_of(&tokens), vec![1, 1, 2, 2, 3, 3]);
}
