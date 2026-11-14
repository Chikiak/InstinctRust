//! Diagnostic and error ordering tests (AC6, P2, P4, P7).

use super::*;

// ---------------------------------------------------------------------------
// AC6: unrecognized line and multiple errors ordered by line
// ---------------------------------------------------------------------------

#[test]
fn test_unrecognized_line_reports_its_text() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nfrobnicate\n",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("frobnicate")),
        "should name the unrecognized line text, got: {diags:?}"
    );
}

#[test]
fn test_multiple_errors_ordered_by_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\ngoto missing1\nbogus line\ngoto missing2\n",
    );
    assert!(
        diags.len() >= 3,
        "expected at least 3 diagnostics, got: {diags:?}"
    );
    // Check they're ordered by line number.
    let lines: Vec<usize> = diags.iter().map(|d| d.line).collect();
    assert!(
        lines.windows(2).all(|w| w[0] <= w[1]),
        "diagnostics should be ordered by line, got: {lines:?}"
    );
}

// ---------------------------------------------------------------------------
// AC1: argument parse errors are reported (P2)
// ---------------------------------------------------------------------------

#[test]
fn test_action_arg_error_is_reported() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait(@)\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("unknown character")),
        "argument parse error should be reported, got: {diags:?}"
    );
    assert!(
        diags.iter().any(|d| d.line == 7),
        "diagnostic should carry the action's line, got: {diags:?}"
    );
}

// The rest of the file keeps compiling after a bad argument.
#[test]
fn test_action_arg_error_does_not_abort_file() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait(@)\nwait(1)\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("unknown character")),
        "first error reported, got: {diags:?}"
    );
    assert!(
        !diags.iter().any(|d| d.message.contains("unrecognized")),
        "second action should still compile, got: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// AC4: binary operators are stored unquoted (P4)
// ---------------------------------------------------------------------------

#[test]
fn test_binary_op_text_has_no_quotes() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait(1 + 2)\n",
    );
    assert!(diags.is_empty(), "should compile, got: {diags:?}");

    let (cat, _pc) = default_catalogs();
    let result = compile::compile(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait(1 + 2)\n",
        &cat,
        &PerceptionCatalog::default(),
        "test.ins",
    );
    let action = result
        .program
        .lines
        .iter()
        .find_map(|l| match l {
            Line::Action { args, .. } => args.first().map(|a| a.as_ref()),
            _ => None,
        })
        .expect("wait(1 + 2) should compile to an action");
    assert_eq!(
        action.operator(),
        "+",
        "operator should be bare '+', not quoted, got: {:?}",
        action.operator()
    );

    let (diags2, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif 1 == 2 goto start\n",
    );
    assert!(
        diags2.is_empty(),
        "== comparison should compile, got: {diags2:?}"
    );
}

// ---------------------------------------------------------------------------
// P7: additional header/body coverage
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_errors_ordered_by_line_with_header_and_body() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth diez\nvision 3\nlifespan 100\nstart:\nwait(@)\nbogus\n",
    );
    assert!(
        diags.len() >= 3,
        "expected at least 3 diagnostics, got: {diags:?}"
    );
    let lines: Vec<usize> = diags.iter().map(|d| d.line).collect();
    assert!(
        lines.windows(2).all(|w| w[0] <= w[1]),
        "diagnostics should be ordered by line, got: {lines:?}"
    );
}
