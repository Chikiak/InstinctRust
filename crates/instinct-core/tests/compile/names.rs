//! Name and arity error tests (AC5, P3, AC8).

use super::*;

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
    // `see` is now a valid function; use an unknown function name instead.
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif bogus(0, 0) == 0 goto start\n",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("unknown function")),
        "should report unknown function 'bogus', got: {diags:?}"
    );
    assert!(
        diags.iter().any(|d| d.line == 7),
        "diagnostic should carry the line number, got: {diags:?}"
    );
}

#[test]
fn test_compile_rejects_unknown_function() {
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif nonexistent(1, 2) goto start\n",
    );
    assert!(
        diags.iter().any(|d| d.message.contains("unknown function")),
        "should report unknown function, got: {diags:?}"
    );
}

#[test]
fn test_compile_rejects_wrong_arity() {
    // see with 1 argument (wrong arity)
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif see(1) goto start\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("see") && d.message.contains("arguments")),
        "should report wrong arity for 'see', got: {diags:?}"
    );

    // see with 0 arguments
    let (diags, _) = compile_src(
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nif see() goto start\n",
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("see") && d.message.contains("arguments")),
        "should report wrong arity for 'see()', got: {diags:?}"
    );
}

#[test]
fn test_compile_rejects_assignment_to_function_and_constant() {
    for name in ["see", "name", "NONE", "GROUND", "OBJECT", "ALLY", "ENEMY"] {
        let (diags, _) = compile_src(&format!(
            "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\n{name} = 5\n"
        ));
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains(name) && d.message.contains("assign")),
            "assigning to {name} should be rejected, got: {diags:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// P3: digit-leading name and `==`
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
// AC8: assigning to any of the ten new names is a compile error
// ---------------------------------------------------------------------------

#[test]
fn test_compile_rejects_assignment_to_new_perceptions() {
    for name in [
        "vision",
        "species",
        "faction",
        "terrain_here",
        "reserve_here",
        "width",
        "height",
        "total_creatures",
        "total_allies",
        "total_enemies",
    ] {
        let (diags, _) = compile_src(&format!(
            "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\n{name} = 5\n"
        ));
        assert!(
            diags
                .iter()
                .any(|d| d.message.contains(name) && d.message.contains("perception")),
            "assigning to {name} should be rejected as a perception, got: {diags:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// P7: additional header/body coverage
// ---------------------------------------------------------------------------

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
