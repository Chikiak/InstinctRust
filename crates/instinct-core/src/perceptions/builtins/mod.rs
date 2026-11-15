//! Built-in perceptions: unit structs implementing [`Perception`].
//!
//! Organized by semantic family: core creature state, near-vision proximity,
//! and social/world scope. All types are re-exported here so
//! `perceptions/mod.rs` and `catalog.rs` keep seeing exactly what they saw.

pub mod core;
pub mod near;
pub mod social;

pub use core::{Age, Health, Lifespan, Tick, Vision, X, Y};
pub use near::{
    AlliesNear, AllyDist, AllyDx, AllyDy, EnemiesNear, EnemyDist, EnemyDx, EnemyDy, ObjectDist,
    ObjectDx, ObjectDy, ObjectsNear,
};
pub use social::{
    Faction, Height, LastRoar, Random, ReserveHere, RoarsNear, Species, TerrainHere, TotalAllies,
    TotalCreatures, TotalEnemies, Width,
};
