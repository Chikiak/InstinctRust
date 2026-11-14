//! Read-only view of the world and the creature currently being evaluated.
//! Nothing here mutates the world; `random` is the only operation that
//! advances state, and it does so through interior mutability confined to
//! the generator itself.

use crate::perceptions::nearby::Nearby;

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

    /// Vision range of the creature's species (header attribute).
    fn vision(&self) -> i64;
    /// Name of the creature's species.
    fn species_name(&self) -> &str;
    /// Faction of the creature's species.
    fn faction_name(&self) -> &str;
    /// Name of the terrain under the creature.
    fn terrain_here(&self) -> &str;
    /// Current reserve of the terrain under the creature.
    fn reserve_here(&self) -> i64;
    /// Map width in columns.
    fn width(&self) -> i64;
    /// Map height in rows.
    fn height(&self) -> i64;
    /// Living creatures in the world except this one.
    fn total_creatures(&self) -> i64;
    /// Living allies (same faction) except this one.
    fn total_allies(&self) -> i64;
    /// Living enemies (other faction).
    fn total_enemies(&self) -> i64;

    /// Nearby creatures, objects, and roars within the creature's vision radius.
    /// Recalculated on every read; no caching.
    fn nearby(&self) -> Nearby;

    /// Content constant for the cell at (x, y): -1 NONE out of radius,
    /// 0 GROUND, 1 OBJECT, 2 ALLY, 3 ENEMY. Radius checked first.
    fn see(&self, x: i64, y: i64) -> i64;

    /// Name of the element at (x, y): species name, object name, terrain
    /// name, or empty text out of radius/map.
    fn name(&self, x: i64, y: i64) -> &str;
}
