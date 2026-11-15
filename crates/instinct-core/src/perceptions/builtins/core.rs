//! Built-in perceptions: creature state (health, age, lifespan, position, tick, vision).

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
