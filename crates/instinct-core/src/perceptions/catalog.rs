//! Catalog of perceptions by name. Resolves a perception name without arity
//! validation (perceptions take no arguments).

use crate::diagnostic::Diagnostic;
use crate::perceptions::builtins;
use crate::perceptions::Perception;

/// Catalog of perceptions by name. Resolves a perception name without arity
/// validation (perceptions take no arguments).
#[derive(Debug)]
pub struct PerceptionCatalog {
    perceptions: Vec<Box<dyn Perception>>,
}

impl Clone for PerceptionCatalog {
    fn clone(&self) -> Self {
        PerceptionCatalog {
            perceptions: self.perceptions.iter().map(|p| p.clone_box()).collect(),
        }
    }
}

impl PerceptionCatalog {
    pub fn new() -> Self {
        PerceptionCatalog {
            perceptions: Vec::new(),
        }
    }

    /// Registers a perception type. Returns an error if a perception with the
    /// same name is already registered.
    pub fn register(&mut self, perception: Box<dyn Perception>) -> Result<(), Diagnostic> {
        if self
            .perceptions
            .iter()
            .any(|p| p.name() == perception.name())
        {
            return Err(Diagnostic::new(
                "catalog",
                1,
                format!("perception '{}' already registered", perception.name()),
            ));
        }
        self.perceptions.push(perception);
        Ok(())
    }

    /// Resolves a perception by name, borrowing it from the catalog.
    pub fn resolve(&self, name: &str) -> Option<&dyn Perception> {
        self.perceptions
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    /// Returns true if `name` is a registered perception.
    pub fn contains(&self, name: &str) -> bool {
        self.perceptions.iter().any(|p| p.name() == name)
    }
}

impl Default for PerceptionCatalog {
    fn default() -> Self {
        let mut c = PerceptionCatalog::new();
        let _ = c.register(Box::new(builtins::Health));
        let _ = c.register(Box::new(builtins::Age));
        let _ = c.register(Box::new(builtins::Lifespan));
        let _ = c.register(Box::new(builtins::X));
        let _ = c.register(Box::new(builtins::Y));
        let _ = c.register(Box::new(builtins::Tick));
        let _ = c.register(Box::new(builtins::Random));
        let _ = c.register(Box::new(builtins::Vision));
        let _ = c.register(Box::new(builtins::Species));
        let _ = c.register(Box::new(builtins::Faction));
        let _ = c.register(Box::new(builtins::TerrainHere));
        let _ = c.register(Box::new(builtins::ReserveHere));
        let _ = c.register(Box::new(builtins::Width));
        let _ = c.register(Box::new(builtins::Height));
        let _ = c.register(Box::new(builtins::TotalCreatures));
        let _ = c.register(Box::new(builtins::TotalAllies));
        let _ = c.register(Box::new(builtins::TotalEnemies));
        let _ = c.register(Box::new(builtins::AlliesNear));
        let _ = c.register(Box::new(builtins::EnemiesNear));
        let _ = c.register(Box::new(builtins::ObjectsNear));
        let _ = c.register(Box::new(builtins::EnemyDist));
        let _ = c.register(Box::new(builtins::EnemyDx));
        let _ = c.register(Box::new(builtins::EnemyDy));
        let _ = c.register(Box::new(builtins::AllyDist));
        let _ = c.register(Box::new(builtins::AllyDx));
        let _ = c.register(Box::new(builtins::AllyDy));
        let _ = c.register(Box::new(builtins::ObjectDist));
        let _ = c.register(Box::new(builtins::ObjectDx));
        let _ = c.register(Box::new(builtins::ObjectDy));
        let _ = c.register(Box::new(builtins::RoarsNear));
        let _ = c.register(Box::new(builtins::LastRoar));
        c
    }
}
