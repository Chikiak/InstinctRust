//! Evaluation context for a single creature.

use crate::expr::EvalContext;
use crate::perceptions::CreatureView;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::creature::CreatureId;
use super::snapshot::CreatureSnapshot;
use super::World;

/// Evaluation context for a single creature. Resolves perceptions first, then
/// variables. Execution reads the world immutably; the world applies actions
/// afterward. Holds a snapshot of the creature for function calls and a handle
/// to the turn-state variables so assignments made during the same turn are
/// visible to subsequent expressions.
pub struct CreatureContext<'a> {
    world: &'a World,
    creature_id: CreatureId,
    snapshot: CreatureSnapshot<'a>,
    variables: Rc<RefCell<HashMap<String, Value>>>,
}

impl<'a> CreatureContext<'a> {
    pub fn new(
        world: &'a World,
        creature_id: CreatureId,
        variables: Rc<RefCell<HashMap<String, Value>>>,
    ) -> Self {
        let snapshot = CreatureSnapshot { world, creature_id };
        CreatureContext {
            world,
            creature_id,
            snapshot,
            variables,
        }
    }
}

impl<'a> EvalContext for CreatureContext<'a> {
    fn resolve(&self, name: &str) -> Option<Value> {
        // Perceptions take priority over variables.
        if let Some(perception) = self.world.catalog.resolve_perception(name) {
            return Some(perception.read(&self.snapshot));
        }
        // Constants resolve next.
        if let Some(value) = self.world.catalog.resolve_constant(name) {
            return Some(Value::Int(value));
        }
        // Turn-state variables (assignments made this turn) shadow persisted
        // variables stored on the creature in the world.
        self.variables
            .borrow()
            .get(name)
            .cloned()
            .or_else(|| self.world.variable(self.creature_id, name))
    }

    fn view(&self) -> Option<&dyn CreatureView> {
        Some(&self.snapshot)
    }
}
