use instinct_core::value::Value;

#[test]
fn test_value_is_truthy() {
    assert!(!Value::Int(0).is_truthy());
    assert!(Value::Int(1).is_truthy());
    assert!(Value::Int(-1).is_truthy());
    assert!(!Value::Text("".to_string()).is_truthy());
    assert!(Value::Text("a".to_string()).is_truthy());
}
