//! Perceptions: read-only traits that expose creature and world state to
//! expressions.
//!
//! Perceptions are registered by name in the catalog and resolved by name,
//    not by a chain of `if nombre == ...` per name.

use crate::diagnostic::Diagnostic;
use crate::value::Value;

/// A read-only snapshot of the world and the creature currently being
/// evaluated. Nothing here mutates the world; `random` is the only operation
/// that advances state, and it does so through interior mutability confined to
/// the generator itself.
pub trait CreatureView: std::fmt::Debug {
    /// Current health of the creature.
    fn health(&self) -> i64;
    /// Current age of the creature.
    fn age(&self) -> i64;
    /// Maximum lifespan of the creature's species.
    fn lifespan(&self) -> i64;
    /// X coordinate on the map.
    fn x(&self) -> i64;
    /// Y coordinate on the map.
    fn y(&self) -> i64;
    /// Number of completed ticks in the world.
    fn tick(&self) -> u64;
    /// A random integer between 0 and 99. May advance the generator.
    fn random(&self) -> i64;
}

/// Trait every perception type implements.
pub trait Perception: std::fmt::Debug {
    /// Perception name as it appears in the language (e.g. `health`, `random`).
    fn name(&self) -> &str;
    /// Reads the perception value from `view`.
    fn read(&self, view: &dyn CreatureView) -> Value;
    /// Clones the perception into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Perception>;
}

/// Health of the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health;

impl Perception for Health {
    fn name(&self) -> &str {
        "health"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.health())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Age of the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Age;

impl Perception for Age {
    fn name(&self) -> &str {
        "age"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.age())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Maximum lifespan of the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lifespan;

impl Perception for Lifespan {
    fn name(&self) -> &str {
        "lifespan"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.lifespan())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// X coordinate of the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X;

impl Perception for X {
    fn name(&self) -> &str {
        "x"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.x())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Y coordinate of the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Y;

impl Perception for Y {
    fn name(&self) -> &str {
        "y"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.y())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Number of completed ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tick;

impl Perception for Tick {
    fn name(&self) -> &str {
        "tick"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.tick() as i64)
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// A random integer between 0 and 99. Advances the world's generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Random;

impl Perception for Random {
    fn name(&self) -> &str {
        "random"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.random())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

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
        let _ = c.register(Box::new(Health));
        let _ = c.register(Box::new(Age));
        let _ = c.register(Box::new(Lifespan));
        let _ = c.register(Box::new(X));
        let _ = c.register(Box::new(Y));
        let _ = c.register(Box::new(Tick));
        let _ = c.register(Box::new(Random));
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct FixedView;

    impl CreatureView for FixedView {
        fn health(&self) -> i64 {
            10
        }
        fn age(&self) -> i64 {
            3
        }
        fn lifespan(&self) -> i64 {
            50
        }
        fn x(&self) -> i64 {
            4
        }
        fn y(&self) -> i64 {
            2
        }
        fn tick(&self) -> u64 {
            7
        }
        fn random(&self) -> i64 {
            5
        }
    }

    #[test]
    fn test_perception_default_catalog_contains_seven() {
        let cat = PerceptionCatalog::default();
        for name in ["health", "age", "lifespan", "x", "y", "tick", "random"] {
            assert!(cat.contains(name), "perception {name} registered");
        }
        assert!(cat.resolve("nope").is_none());
    }

    #[test]
    fn test_perception_reads_creature_state() {
        let cat = PerceptionCatalog::default();
        let view = FixedView;
        let cases: &[(&str, i64)] = &[
            ("health", 10),
            ("age", 3),
            ("lifespan", 50),
            ("x", 4),
            ("y", 2),
            ("tick", 7),
            ("random", 5),
        ];
        for (name, expected) in cases {
            let p = cat.resolve(name).expect("perception resolves");
            assert_eq!(p.read(&view), Value::Int(*expected), "{name}");
        }
    }

    #[test]
    fn test_perception_rejects_unknown_name() {
        let cat = PerceptionCatalog::default();
        assert!(cat.resolve("not_a_perception").is_none());
    }
}
