//! Unit tests for `.ins` compilation (F0-06): header, body, labels, names,
//! arity, and diagnostics.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::perception::PerceptionCatalog;
use instinct_core::program::Line;

fn default_catalogs() -> (Catalog, PerceptionCatalog) {
    (Catalog::default(), PerceptionCatalog::default())
}

fn compile_src(
    text: &str,
) -> (
    Vec<instinct_core::diagnostic::Diagnostic>,
    Option<instinct_core::compile::Header>,
) {
    let (cat, pc) = default_catalogs();
    let result = compile::compile(text, &cat, &pc, "test.ins");
    (result.diagnostics, result.header)
}

// ---------------------------------------------------------------------------
// AC1: walker.ins compiles
// ---------------------------------------------------------------------------

#[test]
fn test_compile_walker_example() {
    let path = format!(
        "{}/../../examples/creatures/walker.ins",
        env!("CARGO_MANIFEST_DIR")
    );
    let content = std::fs::read_to_string(&path).expect("walker.ins must exist");
    let (cat, pc) = default_catalogs();
    let result = compile::compile(&content, &cat, &pc, &path);
    assert!(
        result.diagnostics.is_empty(),
        "walker.ins should compile without diagnostics, got: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("walker.ins should have a header");
    assert_eq!(header.species_name, "Walker");
    assert_eq!(header.faction, "any");
    assert_eq!(header.health, 10);
    assert_eq!(header.vision, 3);
    assert_eq!(header.lifespan, 100);
    assert!(
        result.program.resolve_label("start").is_some(),
        "program should have a 'start' label"
    );
}

// ---------------------------------------------------------------------------
// AC2: header validation
// ---------------------------------------------------------------------------

#[test]
fn test_header_keys_in_any_order() {
    let (diags, header) =
        compile_src("creature X\nlifespan 100\nvision 3\nhealth 10\nfaction any\nstart:\n");
    assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");
    let h = header.expect("header should be parsed");
    assert_eq!(h.species_name, "X");
    assert_eq!(h.faction, "any");
    assert_eq!(h.health, 10);
    assert_eq!(h.vision, 3);
    assert_eq!(h.lifespan, 100);
}

#[test]
fn test_header_missing_key_reports_on_line_1() {
    let (diags, _) = compile_src("creature X\nfaction any\nhealth 10\nvision 3\nstart:\n");
    assert!(
        diags
            .iter()
            .any(|d| d.line == 1 && d.message.contains("lifespan")),
        "should report missing 'lifespan' on line 1, got: {diags:?}"
    );
}

#[test]
fn test_header_repeated_key_reports_on_its_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nfaction other\nstart:\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("repeated header key 'faction'")),
        "should report repeated 'faction', got: {diags:?}"
    );
}

#[test]
fn test_header_unknown_key_reports_on_its_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\ncolor red\nstart:\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("unknown header key 'color'")),
        "should report unknown key 'color', got: {diags:?}"
    );
}

#[test]
fn test_header_non_integer_value_reports_on_its_line() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth diez\nvision 3\nlifespan 100\nstart:\n");
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("health") && d.message.contains("integer")),
        "should report non-integer health, got: {diags:?}"
    );
}

#[test]
fn test_header_zero_health_reports_on_its_line() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 0\nvision 3\nlifespan 100\nstart:\n");
    assert!(
        diags.iter().any(|d| d.message.contains("health")),
        "should report health 0, got: {diags:?}"
    );
}

#[test]
fn test_header_vision_zero_reports_on_its_line() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 10\nvision 0\nlifespan 100\nstart:\n");
    assert!(
        diags.iter().any(|d| d.message.contains("vision")),
        "should report vision 0, got: {diags:?}"
    );
}

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
// AC4: label errors
// ---------------------------------------------------------------------------

#[test]
fn test_goto_to_missing_label_reports_on_goto_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\ngoto missing\n",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("missing")),
        "should name the missing label, got: {diags:?}"
    );
}

#[test]
fn test_duplicate_label_reports_on_second() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nfoo:\ngoto start\nfoo:\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("duplicate label 'foo'")),
        "should report duplicate 'foo', got: {diags:?}"
    );
}

#[test]
fn test_missing_start_reports_on_line_1() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\ngoto start\n");
    assert!(
        diags
            .iter()
            .any(|d| d.line == 1 && d.message.contains("start")),
        "should report missing 'start' on line 1, got: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// AC5: name and arity errors
// ---------------------------------------------------------------------------

#[test]
fn test_assign_to_perception_reports_on_its_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nhealth = 5\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("health") && d.message.contains("perception")),
        "should report assigning to perception 'health', got: {diags:?}"
    );
}

#[test]
fn test_assign_to_action_reports_on_its_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait = 5\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("wait") && d.message.contains("action")),
        "should report assigning to action 'wait', got: {diags:?}"
    );
}

#[test]
fn test_assign_to_reserved_word_reports_on_its_line() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif = 5\n");
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("if") && d.message.contains("reserved")),
        "should report assigning to reserved word 'if', got: {diags:?}"
    );
}

#[test]
fn test_unknown_action_reports_on_its_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nfly(1, 2)\n",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("fly")),
        "should report unknown action 'fly', got: {diags:?}"
    );
}

#[test]
fn test_wrong_arity_reports_on_its_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait(1, 2)\n",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("wait")),
        "should report wrong arity for 'wait', got: {diags:?}"
    );
}

#[test]
fn test_function_in_expression_reports_on_its_line() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif see(0, 0) == 0 goto start\n",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("function")),
        "should report function in expression, got: {diags:?}"
    );
}

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
// AC2: assignment to a digit-leading name is rejected (P3)
// ---------------------------------------------------------------------------

#[test]
fn test_assignment_digit_name_rejected() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\n1x = 5\n");
    assert!(
        !diags.is_empty(),
        "digit-leading name should be rejected, got: {diags:?}"
    );
    assert!(
        !diags.iter().any(|d| d.message.contains("assign")),
        "should not be treated as a (valid) assignment, got: {diags:?}"
    );
}

// ---------------------------------------------------------------------------
// AC3: `==` is not treated as assignment (P3)
// ---------------------------------------------------------------------------

#[test]
fn test_double_equals_not_assignment() {
    // `==` in a comparison is an operator, not an assignment.
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif x == 5 goto start\n",
    );
    assert!(
        diags.is_empty(),
        "comparison should compile, got: {diags:?}"
    );

    // A bare `==` line is not treated as an assignment.
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nx == 5\n");
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("unrecognized line")),
        "bare == should be unrecognized, not an assignment, got: {diags:?}"
    );
    assert!(
        !diags.iter().any(|d| d.message.contains("assign")),
        "should not be treated as assignment, got: {diags:?}"
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
// AC5: invalid header value produces exactly one diagnostic (P5)
// ---------------------------------------------------------------------------

#[test]
fn test_header_invalid_value_single_diagnostic() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth abc\nvision 3\nlifespan 100\nstart:\n");
    let health_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("health"))
        .collect();
    assert_eq!(
        health_diags.len(),
        1,
        "exactly one diagnostic for invalid health, got: {diags:?}"
    );
    assert_eq!(
        health_diags[0].line, 3,
        "on the value's line, got: {diags:?}"
    );
    assert!(
        health_diags[0].message.contains("integer"),
        "should name the integer problem, got: {diags:?}"
    );
    assert_eq!(diags.len(), 1, "no other diagnostics, got: {diags:?}");
}

// ---------------------------------------------------------------------------
// P7: additional header/body coverage
// ---------------------------------------------------------------------------

#[test]
fn test_header_negative_value_reports_on_its_line() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth -5\nvision 3\nlifespan 100\nstart:\n");
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("health") && d.message.contains("-5")),
        "should report negative health, got: {diags:?}"
    );
}

#[test]
fn test_header_lifespan_zero_reports_on_its_line() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 10\nvision 3\nlifespan 0\nstart:\n");
    assert!(
        diags.iter().any(|d| d.message.contains("lifespan")),
        "should report lifespan 0, got: {diags:?}"
    );
}

#[test]
fn test_header_uppercase_key_is_unknown() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nHealth 10\nstart:\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("unknown header key 'Health'")),
        "should report unknown uppercase key, got: {diags:?}"
    );
}

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

#[test]
fn test_action_with_empty_args_reports_arity() {
    let (diags, _) =
        compile_src("creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nwait()\n");
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("wait") && d.message.contains("arguments")),
        "empty args should report arity, got: {diags:?}"
    );
}

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
