//! Body line form tests (AC3, P7).

use super::*;

// ---------------------------------------------------------------------------
// AC3: body line forms
// ---------------------------------------------------------------------------

#[test]
fn test_body_all_line_forms() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\n# comment\n\n   \tgoto loop\nif health > 0 goto loop\nenergy = 5\nwait(1)\nloop:\n",
    );
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
}

// ---------------------------------------------------------------------------
// P7: additional header/body coverage
// ---------------------------------------------------------------------------

#[test]
fn test_goto_inside_text_is_not_a_statement() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nsay(\"goto start\")\n",
    );
    assert!(
        diags.is_empty(),
        "goto inside text should compile, got: {diags:?}"
    );
}
