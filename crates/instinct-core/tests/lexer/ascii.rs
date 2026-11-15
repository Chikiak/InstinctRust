//! Identifiers are ASCII; non-ASCII characters are errors.

use instinct_core::lexer::lex;

#[test]
fn test_lex_non_ascii_name_errors() {
    let err = lex("naïve", 1, "test.ins").unwrap_err();
    assert_eq!(err.line, 1);
    assert!(err.message.contains('ï'));
}
