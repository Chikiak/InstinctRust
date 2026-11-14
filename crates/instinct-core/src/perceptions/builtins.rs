//! Built-in perceptions: unit structs implementing [`Perception`].

use crate::perceptions::{CreatureView, Perception};
use crate::value::Value;

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
