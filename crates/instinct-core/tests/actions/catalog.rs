//! Catalog action tests.

use super::common::*;
use instinct_core::actions::{Action, Catalog};
use instinct_core::value::Value;
use std::rc::Rc;

#[test]
fn test_catalog_resolves_three_actions_with_arity() {
    let cat = Catalog::default();
    let wait = cat.resolve("wait", 1).expect("wait arity 1");
    assert_eq!(wait.name(), "wait");
    assert_eq!(wait.arity(), 1);
    assert!(wait.consumes_turn());

    let mv = cat.resolve("move", 3).expect("move arity 3");
    assert_eq!(mv.name(), "move");
    assert_eq!(mv.arity(), 3);
    assert!(mv.consumes_turn());

    let say = cat.resolve("say", 1).expect("say arity 1");
    assert_eq!(say.name(), "say");
    assert_eq!(say.arity(), 1);
    assert!(!say.consumes_turn());
}

#[test]
fn test_catalog_rejects_unknown_action_and_wrong_arity() {
    let cat = Catalog::default();
    assert!(cat.resolve("nope", 0).is_err());
    assert!(cat.resolve("wait", 2).is_err());
    assert!(cat.resolve("move", 2).is_err());
    assert!(cat.resolve("say", 0).is_err());
}

#[test]
fn test_catalog_extension_with_test_only_action() {
    // A test-defined action type not in the existing sources.
    #[derive(Debug)]
    struct TestAction;

    impl Action for TestAction {
        fn name(&self) -> &str {
            "test_action"
        }
        fn arity(&self) -> usize {
            0
        }
        fn consumes_turn(&self) -> bool {
            false
        }
        fn apply(
            &self,
            _world: &mut World,
            _creature_id: CreatureId,
            _args: &[Value],
        ) -> Result<ActionResult, Diagnostic> {
            Ok(ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            })
        }
    }

    let mut cat = Catalog::default();
    assert!(cat.resolve("test_action", 0).is_err());
    assert!(
        cat.register(Rc::new(TestAction)).is_ok(),
        "register the test action"
    );
    let ta = cat.resolve("test_action", 0).expect("resolve test_action");
    assert_eq!(ta.name(), "test_action");
    assert_eq!(ta.arity(), 0);
    // Existing actions still resolve.
    assert!(cat.resolve("wait", 1).is_ok());
    assert!(cat.resolve("move", 3).is_ok());
    assert!(cat.resolve("say", 1).is_ok());
}
