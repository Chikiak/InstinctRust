//! Unterminated text literals and interior newlines.

use instinct_core::lexer::lex;

#[test]
fn test_lex_unterminated_text() {
    let err = lex("\"abc", 1, "test.ins").unwrap_err();
    assert_eq!(err.line, 1);
    assert_eq!(err.message, "unterminated text literal");

    // A newline inside the text is an error reported on the newline's line.
    let err = lex("\"a\n", 1, "test.ins").unwrap_err();
    assert_eq!(err.line, 2);
    assert_eq!(err.message, "unterminated text literal");
}

#[test]
fn test_lex_text_with_interior_newline_is_error() {
    let err = lex("\"a\nb\"", 1, "test.ins").unwrap_err();
    assert_eq!(err.line, 2);
    assert_eq!(err.message, "unterminated text literal");
}
