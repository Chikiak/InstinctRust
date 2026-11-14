//! Perceptions: read-only traits that expose creature and world state to
//! expressions.
//!
//! Perceptions are registered by name in the catalog and resolved by name,
//! not by a chain of `if name == ...` per name.

use crate::value::Value;

pub mod builtins;
pub mod catalog;
pub mod nearby;
pub mod view;

pub trait Perception: std::fmt::Debug {
    /// Perception name as it appears in the language (e.g. `health`, `random`).
    fn name(&self) -> &str;
    /// Reads the perception value from `view`.
    fn read(&self, view: &dyn CreatureView) -> Value;
    /// Clones the perception into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Perception>;
}

pub use builtins::{
    AlliesNear, AllyDist, AllyDx, AllyDy, Age, EnemyDist, EnemyDx, EnemyDy,
    EnemiesNear, Faction, Health, Height, LastRoar, Lifespan, ObjectDist,
    ObjectDx, ObjectDy, ObjectsNear, Random, ReserveHere, RoarsNear, Species,
    Tick, TotalAllies, TotalCreatures, TotalEnemies, TerrainHere, Vision,
    Width, X, Y,
};
pub use catalog::PerceptionCatalog;
pub use nearby::{Nearest, Nearby};
pub use view::CreatureView;

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
        fn vision(&self) -> i64 {
            1
        }
        fn species_name(&self) -> &str {
            "Fixed"
        }
        fn faction_name(&self) -> &str {
            "Neutral"
        }
        fn terrain_here(&self) -> &str {
            "Grass"
        }
        fn reserve_here(&self) -> i64 {
            42
        }
        fn width(&self) -> i64 {
            10
        }
        fn height(&self) -> i64 {
            10
        }
        fn total_creatures(&self) -> i64 {
            3
        }
        fn total_allies(&self) -> i64 {
            1
        }
        fn total_enemies(&self) -> i64 {
            2
        }

        fn nearby(&self) -> Nearby {
            Nearby::empty()
        }

        fn see(&self, _x: i64, _y: i64) -> i64 {
            0
        }

        fn name(&self, _x: i64, _y: i64) -> &str {
            ""
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
