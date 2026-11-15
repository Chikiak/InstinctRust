//! Built-in perceptions: species, faction, terrain, reserve, map size and
//! social/world scope (totals, roars, random).

use crate::perceptions::{CreatureView, Perception};
use crate::value::Value;

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
