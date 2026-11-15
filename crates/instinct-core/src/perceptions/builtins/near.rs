//! Built-in perceptions: near-vision proximity and distances to allies,
//! enemies and objects.

use crate::perceptions::{CreatureView, Perception};
use crate::value::Value;

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
