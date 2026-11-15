//! Tests for `Program::resolve_label` with known and unknown labels.

use super::compile_text;
use instinct_core::program::Program;

#[test]
fn test_resolve_label_known_and_unknown() {
    let text =
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\ngoto end\nend:\n";
    let result = compile_text(text);
    assert!(
        result.diagnostics.is_empty(),
        "unexpected diagnostics: {:?}",
        result.diagnostics
    );

    // Indices into `lines` (only the body is stored in `lines`):
    // 0 = Label start, 1 = Goto end, 2 = Label end
    assert_eq!(result.program.resolve_label("start"), Some(0));
    assert_eq!(result.program.resolve_label("end"), Some(2));
    assert_eq!(result.program.resolve_label("no_existe"), None);

    // Empty program: no labels.
    assert_eq!(Program::new().resolve_label("cualquiera"), None);
    assert_eq!(Program::default().resolve_label("cualquiera"), None);
}
