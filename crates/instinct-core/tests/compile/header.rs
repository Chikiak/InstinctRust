//! Header validation tests (AC2, P5, P7).

use super::*;

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
// P5: invalid header value produces exactly one diagnostic
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
