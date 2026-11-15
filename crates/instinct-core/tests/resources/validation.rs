//! Resource validation diagnostics tests for the `resources` module.
use super::common::*;

#[test]
fn test_resource_validation_diagnostics() {
    // Missing key
    let (_, d) = load_terrain("terrain X\nchar .\nregen 2\n", "missing.ob");
    assert!(
        d.iter().any(|diag| diag.message.contains("resource_max")),
        "missing resource_max: {d:?}"
    );
    // Non-numeric value
    let (_, d) = load_object("object X\nchar #\nresource_max abc\n", "number.ob");
    assert!(
        d.iter().any(|diag| diag.message.contains("resource_max")),
        "non-numeric value: {d:?}"
    );
    // char with more than one character
    let (_, d) = load_terrain("terrain X\nchar ab\nresource_max 10\nregen 1\n", "char.ob");
    assert!(
        d.iter().any(|diag| diag.message.contains("char")),
        "multi-char: {d:?}"
    );
    // resource_max <= 0
    let (_, d) = load_terrain("terrain X\nchar .\nresource_max 0\nregen 1\n", "max.ob");
    assert!(!d.is_empty(), "resource_max 0 must fail: {d:?}");
    // regen < 0
    let (_, d) = load_terrain("terrain X\nchar .\nresource_max 10\nregen -1\n", "regen.ob");
    assert!(!d.is_empty(), "regen < 0 must fail: {d:?}");
}
