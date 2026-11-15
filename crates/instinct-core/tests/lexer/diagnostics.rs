//! Error-to-diagnostic conversion and error line reporting.

use instinct_core::diagnostic::Diagnostic;
use instinct_core::lexer::lex;

#[test]
fn test_lex_error_to_diagnostic() {
    // LexError carries file, line and message; conversion to Diagnostic preserves them.
    let err = lex("x @ 1", 5, "entrada.ins").unwrap_err();
    let diag: Diagnostic = err.into();
    assert_eq!(diag.file, "entrada.ins");
    assert_eq!(diag.line, 5);
    assert!(diag.message.contains('@'));
}

#[test]
fn test_lexer_unknown_character_reports_line() {
    let err = lex("x @ 1", 5, "test.ins").unwrap_err();
    assert_eq!(err.line, 5);
    assert!(err.message.contains('@'));
}

#[test]
fn test_lexer_unterminated_string_reports_line() {
    let err = lex("unterminated \"", 2, "test.ins").unwrap_err();
    assert_eq!(err.line, 2);
}
