//! Variable perception tests.

use super::common::*;
use instinct_core::value::Value;

#[test]
fn test_variables_assign_read_and_retype() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    world
        .set_variable(id, "score", Value::Int(41))
        .expect("assign score");
    assert_eq!(eval(&world, id, "score").unwrap(), Value::Int(41));

    // A variable can change from integer to text on reassignment.
    world
        .set_variable(id, "score", Value::Text("listo".to_string()))
        .expect("reassign score");
    assert_eq!(
        eval(&world, id, "score").unwrap(),
        Value::Text("listo".to_string())
    );
}

#[test]
fn test_unassigned_variable_error() {
    let mut world = empty_world();
    let id = place_walker(&mut world);
    let err = eval(&world, id, "missing_var").unwrap_err();
    assert!(
        err.contains("missing_var"),
        "error names the variable: {err}"
    );
}
