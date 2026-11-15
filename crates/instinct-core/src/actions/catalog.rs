//! `Catalog`: actions and perceptions by name. Resolves each kind of name
//! against its own registry, validating arity for actions.

use crate::actions::action_catalog::ActionCatalog;
use crate::diagnostic::Diagnostic;
use crate::functions::{Function, FunctionCatalog};
use crate::perceptions::Perception;
use crate::perceptions::PerceptionCatalog;
use std::rc::Rc;

/// Catalog of actions and perceptions by name. Resolves each kind of name
/// against its own registry, validating arity for actions.
#[derive(Debug, Clone)]
pub struct Catalog {
    actions: ActionCatalog,
    perceptions: PerceptionCatalog,
    functions: FunctionCatalog,
}

impl Catalog {
    pub fn new() -> Self {
        Catalog {
            actions: ActionCatalog::new(),
            perceptions: PerceptionCatalog::new(),
            functions: FunctionCatalog::new(),
        }
    }

    /// Registers an action type. Returns an error if an action with the same
    /// name is already registered.
    pub fn register(&mut self, action: Rc<dyn crate::actions::Action>) -> Result<(), Diagnostic> {
        self.actions.register(action)
    }

    /// Registers a perception type. Returns an error if a perception with the
    /// same name is already registered.
    pub fn register_perception(
        &mut self,
        perception: Box<dyn Perception>,
    ) -> Result<(), Diagnostic> {
        self.perceptions.register(perception)
    }

    /// Resolves an action by name, validating arity. Returns a diagnostic
    /// naming the argument if unknown or wrong arity.
    pub fn resolve(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Result<Rc<dyn crate::actions::Action>, Diagnostic> {
        self.actions.resolve(name, arg_count)
    }

    /// Resolves a perception by name, borrowing it from the catalog. Returns
    /// `None` when the name is not a registered perception.
    pub fn resolve_perception(&self, name: &str) -> Option<&dyn Perception> {
        self.perceptions.resolve(name)
    }

    /// Returns true if `name` is a registered action.
    pub fn is_action_name(&self, name: &str) -> bool {
        self.actions.is_action_name(name)
    }

    /// Returns true if `name` is a registered perception.
    pub fn is_perception_name(&self, name: &str) -> bool {
        self.perceptions.contains(name)
    }

    /// Resolves a function by name, validating arity. Returns a diagnostic
    /// naming the function if unknown or wrong arity.
    pub fn resolve_function(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Result<Rc<dyn Function>, Diagnostic> {
        self.functions.resolve_function(name, arg_count)
    }

    /// Returns true if `name` is a registered function.
    pub fn is_function_name(&self, name: &str) -> bool {
        self.functions.is_function_name(name)
    }

    /// Resolves a constant by name. Returns `None` when the name is not a
    /// registered constant.
    pub fn resolve_constant(&self, name: &str) -> Option<i64> {
        self.functions.resolve_constant(name)
    }

    /// Returns true if `name` is a registered constant.
    pub fn is_constant_name(&self, name: &str) -> bool {
        self.functions.is_constant_name(name)
    }

    /// Returns a reference to the function catalog.
    pub fn functions(&self) -> &FunctionCatalog {
        &self.functions
    }

    /// Returns a reference to the action catalog.
    pub fn actions(&self) -> &ActionCatalog {
        &self.actions
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Catalog {
            actions: ActionCatalog::default(),
            perceptions: PerceptionCatalog::default(),
            functions: FunctionCatalog::default(),
        }
    }
}
