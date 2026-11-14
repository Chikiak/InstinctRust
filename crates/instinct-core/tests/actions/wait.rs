//! Wait action tests.

use super::common::*;
use instinct_core::actions::Wait;
use instinct_core::value::Value;

#[test]
fn test_wait_sleep_turns() {
    let wait = Wait;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);

    // wait 3 -> Sleeping(3).
    let r = wait
        .apply(&mut world, id, &[Value::Int(3)])
        .expect("wait 3");
    assert_eq!(r.sleep_turns, 3);
    assert!(r.events.is_empty());
    assert!(wait.consumes_turn());

    // wait 1 -> Sleeping(1).
    let r = wait
        .apply(&mut world, id, &[Value::Int(1)])
        .expect("wait 1");
    assert_eq!(r.sleep_turns, 1);

    // wait 0 and negative -> Sleeping(1).
    let r = wait
        .apply(&mut world, id, &[Value::Int(0)])
        .expect("wait 0");
    assert_eq!(r.sleep_turns, 1);
    let r = wait
        .apply(&mut world, id, &[Value::Int(-5)])
        .expect("wait -5");
    assert_eq!(r.sleep_turns, 1);
}

#[test]
fn test_wait_text_argument_fails() {
    let wait = Wait;
    let (mut world, _, _) = world_with_grass();
    let id = place_walker(&mut world);
    let err = wait
        .apply(&mut world, id, &[Value::Text("tres".to_string())])
        .unwrap_err();
    assert!(
        err.message.contains("n"),
        "error must name argument 'n', got: {err:?}"
    );
    assert!(err.message.contains("text"));
}
