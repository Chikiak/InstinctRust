//! Catalog of actions by name. Resolves action names against its registry,
//! validating arity for actions.

use crate::actions::Action;
use crate::diagnostic::Diagnostic;
use std::rc::Rc;

use super::{Attack, Consume, Move, Reproduce, Roar, Say, Wait};

/// Catalog of actions by name. Resolves action names against its registry,
/// validating arity for actions.
#[derive(Debug, Clone)]
pub struct ActionCatalog {
    actions: Vec<Rc<dyn Action>>,
}

impl ActionCatalog {
    pub fn new() -> Self {
        ActionCatalog {
            actions: Vec::new(),
        }
    }

    /// Registers an action type. Returns an error if an action with the same
    /// name is already registered.
    pub fn register(&mut self, action: Rc<dyn Action>) -> Result<(), Diagnostic> {
        if self.actions.iter().any(|a| a.name() == action.name()) {
            return Err(Diagnostic::new(
                "catalog",
                1,
                format!("action '{}' already registered", action.name()),
            ));
        }
        self.actions.push(action);
        Ok(())
    }

    /// Resolves an action by name, validating arity. Returns a diagnostic
    /// naming the argument if unknown or wrong arity.
    pub fn resolve(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Result<Rc<dyn Action>, Diagnostic> {
        match self.actions.iter().find(|a| a.name() == name) {
            Some(action) => {
                if action.arity() != arg_count {
                    return Err(Diagnostic::new(
                        "catalog",
                        1,
                        format!(
                            "action '{}' expects {} arguments, got {}",
                            name,
                            action.arity(),
                            arg_count
                        ),
                    ));
                }
                Ok(Rc::clone(action))
            }
            None => Err(Diagnostic::new(
                "catalog",
                1,
                format!("unknown action '{name}'"),
            )),
        }
    }

    /// Returns true if `name` is a registered action.
    pub fn is_action_name(&self, name: &str) -> bool {
        self.actions.iter().any(|a| a.name() == name)
    }
}

impl Default for ActionCatalog {
    fn default() -> Self {
        let mut c = ActionCatalog::new();
        let _ = c.register(Rc::new(Wait));
        let _ = c.register(Rc::new(Move));
        let _ = c.register(Rc::new(Say));
        let _ = c.register(Rc::new(Attack));
        let _ = c.register(Rc::new(Consume));
        let _ = c.register(Rc::new(Reproduce));
        let _ = c.register(Rc::new(Roar));
        c
    }
}
