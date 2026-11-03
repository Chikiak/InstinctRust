//! Unit tests for the `diagnostic` module.

use instinct_core::diagnostic::Diagnostic;

#[test]
fn test_diagnostic_display_format() {
    let d = Diagnostic::new("creatures/walker.ins", 42, "label 'end' not defined");
    assert_eq!(
        d.to_string(),
        "creatures/walker.ins:42: label 'end' not defined"
    );
    assert_eq!(d.file, "creatures/walker.ins");
    assert_eq!(d.line, 42);
    assert_eq!(d.message, "label 'end' not defined");
}
