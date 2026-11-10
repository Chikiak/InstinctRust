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
}

/// The nearest target of a given class: Chebyshev distance and normalized
/// direction (dx, dy in {-1, 0, 1}).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nearest {
    pub dist: i64,
    pub dx: i64,
    pub dy: i64,
}

/// Aggregated nearby information for a creature: counts, nearest of each
/// class, and roar data from the previous tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nearby {
    pub allies: i64,
    pub enemies: i64,
    pub objects: i64,
    pub ally: Option<Nearest>,
    pub enemy: Option<Nearest>,
    pub object: Option<Nearest>,
    pub roars: i64,
    pub last_roar: String,
}

impl Nearby {
    /// Empty nearby: no allies, enemies, objects, or roars within radius.
    pub fn empty() -> Self {
        Nearby {
            allies: 0,
            enemies: 0,
            objects: 0,
            ally: None,
            enemy: None,
            object: None,
            roars: 0,
            last_roar: String::new(),
        }
    }
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

/// Vision range of the creature's species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vision;

impl Perception for Vision {
    fn name(&self) -> &str {
        "vision"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.vision())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Name of the creature's species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Species;

impl Perception for Species {
    fn name(&self) -> &str {
        "species"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Text(view.species_name().to_string())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Faction of the creature's species.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Faction;

impl Perception for Faction {
    fn name(&self) -> &str {
        "faction"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Text(view.faction_name().to_string())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Name of the terrain under the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainHere;

impl Perception for TerrainHere {
    fn name(&self) -> &str {
        "terrain_here"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Text(view.terrain_here().to_string())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Current reserve of the terrain under the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReserveHere;

impl Perception for ReserveHere {
    fn name(&self) -> &str {
        "reserve_here"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.reserve_here())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Map width in columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Width;

impl Perception for Width {
    fn name(&self) -> &str {
        "width"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.width())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Map height in rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Height;

impl Perception for Height {
    fn name(&self) -> &str {
        "height"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.height())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Living creatures in the world except this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalCreatures;

impl Perception for TotalCreatures {
    fn name(&self) -> &str {
        "total_creatures"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.total_creatures())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Living allies (same faction) except this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalAllies;

impl Perception for TotalAllies {
    fn name(&self) -> &str {
        "total_allies"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.total_allies())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Living enemies (other faction).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TotalEnemies;

impl Perception for TotalEnemies {
    fn name(&self) -> &str {
        "total_enemies"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.total_enemies())
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Number of allied creatures within vision radius, excluding the creature itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlliesNear;

impl Perception for AlliesNear {
    fn name(&self) -> &str {
        "allies_near"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.nearby().allies)
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Number of enemy creatures within vision radius.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemiesNear;

impl Perception for EnemiesNear {
    fn name(&self) -> &str {
        "enemies_near"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.nearby().enemies)
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Number of objects within vision radius.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectsNear;

impl Perception for ObjectsNear {
    fn name(&self) -> &str {
        "objects_near"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.nearby().objects)
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Chebyshev distance to the nearest enemy, or -1 if none visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyDist;

impl Perception for EnemyDist {
    fn name(&self) -> &str {
        "enemy_dist"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().enemy {
            Some(n) => Value::Int(n.dist),
            None => Value::Int(-1),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Normalized x direction (-1, 0, 1) toward the nearest enemy, or 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyDx;

impl Perception for EnemyDx {
    fn name(&self) -> &str {
        "enemy_dx"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().enemy {
            Some(n) => Value::Int(n.dx),
            None => Value::Int(0),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Normalized y direction (-1, 0, 1) toward the nearest enemy, or 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnemyDy;

impl Perception for EnemyDy {
    fn name(&self) -> &str {
        "enemy_dy"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().enemy {
            Some(n) => Value::Int(n.dy),
            None => Value::Int(0),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Chebyshev distance to the nearest ally, or -1 if none visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllyDist;

impl Perception for AllyDist {
    fn name(&self) -> &str {
        "ally_dist"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().ally {
            Some(n) => Value::Int(n.dist),
            None => Value::Int(-1),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Normalized x direction (-1, 0, 1) toward the nearest ally, or 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllyDx;

impl Perception for AllyDx {
    fn name(&self) -> &str {
        "ally_dx"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().ally {
            Some(n) => Value::Int(n.dx),
            None => Value::Int(0),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Normalized y direction (-1, 0, 1) toward the nearest ally, or 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllyDy;

impl Perception for AllyDy {
    fn name(&self) -> &str {
        "ally_dy"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().ally {
            Some(n) => Value::Int(n.dy),
            None => Value::Int(0),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Chebyshev distance to the nearest object, or -1 if none visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectDist;

impl Perception for ObjectDist {
    fn name(&self) -> &str {
        "object_dist"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().object {
            Some(n) => Value::Int(n.dist),
            None => Value::Int(-1),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Normalized x direction (-1, 0, 1) toward the nearest object, or 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectDx;

impl Perception for ObjectDx {
    fn name(&self) -> &str {
        "object_dx"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().object {
            Some(n) => Value::Int(n.dx),
            None => Value::Int(0),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Normalized y direction (-1, 0, 1) toward the nearest object, or 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectDy;

impl Perception for ObjectDy {
    fn name(&self) -> &str {
        "object_dy"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        match view.nearby().object {
            Some(n) => Value::Int(n.dy),
            None => Value::Int(0),
        }
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Number of roars heard from other creatures in the previous tick within vision radius.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoarsNear;

impl Perception for RoarsNear {
    fn name(&self) -> &str {
        "roars_near"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Int(view.nearby().roars)
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

/// Text of the last roar heard from another creature in the previous tick, or empty text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LastRoar;

impl Perception for LastRoar {
    fn name(&self) -> &str {
        "last_roar"
    }
    fn read(&self, view: &dyn CreatureView) -> Value {
        Value::Text(view.nearby().last_roar.clone())
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
        let _ = c.register(Box::new(Vision));
        let _ = c.register(Box::new(Species));
        let _ = c.register(Box::new(Faction));
        let _ = c.register(Box::new(TerrainHere));
        let _ = c.register(Box::new(ReserveHere));
        let _ = c.register(Box::new(Width));
        let _ = c.register(Box::new(Height));
        let _ = c.register(Box::new(TotalCreatures));
        let _ = c.register(Box::new(TotalAllies));
        let _ = c.register(Box::new(TotalEnemies));
        let _ = c.register(Box::new(AlliesNear));
        let _ = c.register(Box::new(EnemiesNear));
        let _ = c.register(Box::new(ObjectsNear));
        let _ = c.register(Box::new(EnemyDist));
        let _ = c.register(Box::new(EnemyDx));
        let _ = c.register(Box::new(EnemyDy));
        let _ = c.register(Box::new(AllyDist));
        let _ = c.register(Box::new(AllyDx));
        let _ = c.register(Box::new(AllyDy));
        let _ = c.register(Box::new(ObjectDist));
        let _ = c.register(Box::new(ObjectDx));
        let _ = c.register(Box::new(ObjectDy));
        let _ = c.register(Box::new(RoarsNear));
        let _ = c.register(Box::new(LastRoar));
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
