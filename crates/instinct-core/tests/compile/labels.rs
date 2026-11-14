//! Label error tests (AC4, P7/A4).

use super::*;

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
// P7 (rest): label table determinism (A4)
// ---------------------------------------------------------------------------

#[test]
fn test_labels_iteration_deterministic() {
    // Expected indices of `lines`: 0=start, 1=zeta, 2=goto alpha,
    // 3=alpha, 4=if goto beta, 5=beta, 6=wait(1).
    let text = "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nzeta:\ngoto alpha\nalpha:\nif health > 0 goto beta\nbeta:\nwait(1)\n";
    let (cat, pc) = default_catalogs();
    let mut previous: Option<Vec<String>> = None;
    for _ in 0..3 {
        let result = compile::compile(text, &cat, &pc, "test.ins");
        assert!(
            result.diagnostics.is_empty(),
            "unexpected diagnostics: {:?}",
            result.diagnostics
        );
        let keys: Vec<String> = result.program.labels.keys().cloned().collect();
        // BTreeMap iterates in label name order.
        assert!(
            keys.windows(2).all(|w| w[0] < w[1]),
            "labels must iterate in order, got: {keys:?}"
        );
        if let Some(prev) = &previous {
            assert_eq!(
                keys.as_slice(),
                prev.as_slice(),
                "label order must be deterministic across compilations"
            );
        }
        previous = Some(keys.clone());
        // resolve_label stable across compilations.
        assert_eq!(result.program.resolve_label("start"), Some(0));
        assert_eq!(result.program.resolve_label("zeta"), Some(1));
        assert_eq!(result.program.resolve_label("alpha"), Some(3));
        assert_eq!(result.program.resolve_label("beta"), Some(5));
        assert_eq!(result.program.resolve_label("no_existe"), None);
    }
}
