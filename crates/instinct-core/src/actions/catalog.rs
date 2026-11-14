//! `Catalog`: actions and perceptions by name. Resolves each kind of name
//! against its own registry, validating arity for actions.

use crate::diagnostic::Diagnostic;
use crate::functions::{Function, FunctionCatalog};
use crate::perceptions::{
    Age, AlliesNear, AllyDist, AllyDx, AllyDy, EnemiesNear, EnemyDist, EnemyDx, EnemyDy, Faction,
    Health, Height, LastRoar, Lifespan, ObjectDist, ObjectDx, ObjectDy, ObjectsNear, Perception,
    PerceptionCatalog, Random, ReserveHere, RoarsNear, Species, TerrainHere, Tick, TotalAllies,
    TotalCreatures, TotalEnemies, Vision, Width, X, Y,
};
use std::rc::Rc;

use super::{Attack, Consume, Move, Reproduce, Roar, Say, Wait};

/// Catalog of actions and perceptions by name. Resolves each kind of name
/// against its own registry, validating arity for actions.
#[derive(Debug, Clone)]
pub struct Catalog {
    actions: Vec<Rc<dyn crate::actions::Action>>,
    perceptions: PerceptionCatalog,
    functions: FunctionCatalog,
}

impl Catalog {
    pub fn new() -> Self {
        Catalog {
            actions: Vec::new(),
            perceptions: PerceptionCatalog::new(),
            functions: FunctionCatalog::new(),
        }
    }

    /// Registers an action type. Returns an error if an action with the same
    /// name is already registered.
    pub fn register(&mut self, action: Rc<dyn crate::actions::Action>) -> Result<(), Diagnostic> {
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

    /// Resolves a perception by name, borrowing it from the catalog. Returns
    /// `None` when the name is not a registered perception.
    pub fn resolve_perception(&self, name: &str) -> Option<&dyn Perception> {
        self.perceptions.resolve(name)
    }

    /// Returns true if `name` is a registered action.
    pub fn is_action_name(&self, name: &str) -> bool {
        self.actions.iter().any(|a| a.name() == name)
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
}

impl Default for Catalog {
    fn default() -> Self {
        let mut c = Catalog::new();
        let _ = c.register(Rc::new(Wait));
        let _ = c.register(Rc::new(Move));
        let _ = c.register(Rc::new(Say));
        let _ = c.register(Rc::new(Attack));
        let _ = c.register(Rc::new(Consume));
        let _ = c.register(Rc::new(Reproduce));
        let _ = c.register(Rc::new(Roar));
        let _ = c.register_perception(Box::new(Health));
        let _ = c.register_perception(Box::new(Age));
        let _ = c.register_perception(Box::new(Lifespan));
        let _ = c.register_perception(Box::new(X));
        let _ = c.register_perception(Box::new(Y));
        let _ = c.register_perception(Box::new(Tick));
        let _ = c.register_perception(Box::new(Random));
        let _ = c.register_perception(Box::new(Vision));
        let _ = c.register_perception(Box::new(Species));
        let _ = c.register_perception(Box::new(Faction));
        let _ = c.register_perception(Box::new(TerrainHere));
        let _ = c.register_perception(Box::new(ReserveHere));
        let _ = c.register_perception(Box::new(Width));
        let _ = c.register_perception(Box::new(Height));
        let _ = c.register_perception(Box::new(TotalCreatures));
        let _ = c.register_perception(Box::new(TotalAllies));
        let _ = c.register_perception(Box::new(TotalEnemies));
        let _ = c.register_perception(Box::new(AlliesNear));
        let _ = c.register_perception(Box::new(EnemiesNear));
        let _ = c.register_perception(Box::new(ObjectsNear));
        let _ = c.register_perception(Box::new(EnemyDist));
        let _ = c.register_perception(Box::new(EnemyDx));
        let _ = c.register_perception(Box::new(EnemyDy));
        let _ = c.register_perception(Box::new(AllyDist));
        let _ = c.register_perception(Box::new(AllyDx));
        let _ = c.register_perception(Box::new(AllyDy));
        let _ = c.register_perception(Box::new(ObjectDist));
        let _ = c.register_perception(Box::new(ObjectDx));
        let _ = c.register_perception(Box::new(ObjectDy));
        let _ = c.register_perception(Box::new(RoarsNear));
        let _ = c.register_perception(Box::new(LastRoar));
        // Register reserved functions and constants.
        let _ = c.functions.register(Rc::new(crate::functions::See));
        let _ = c.functions.register(Rc::new(crate::functions::Name));
        let _ = c.functions.register_constant("NONE", -1);
        let _ = c.functions.register_constant("GROUND", 0);
        let _ = c.functions.register_constant("OBJECT", 1);
        let _ = c.functions.register_constant("ALLY", 2);
        let _ = c.functions.register_constant("ENEMY", 3);
        c
    }
}
