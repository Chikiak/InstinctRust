//! Index conversion tests.

// AC1: integer to index conversion without negative wrapping.
#[test]
fn test_index_from_i64_rejects_negative() {
    assert_eq!(instinct_core::actions::index_from_i64(0), Some(0));
    assert_eq!(instinct_core::actions::index_from_i64(7), Some(7));
    assert_eq!(
        instinct_core::actions::index_from_i64(i64::MAX),
        Some(i64::MAX as usize)
    );
    assert_eq!(instinct_core::actions::index_from_i64(-1), None);
    assert_eq!(instinct_core::actions::index_from_i64(i64::MIN), None);
}
