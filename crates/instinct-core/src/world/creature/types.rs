//! Creature types and memento.

use crate::species::Species;
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Stable identifier of a creature. The world owns creatures by id, never by
/// reference; actions receive an id, not a borrow of the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreatureId(pub usize);

/// An element occupying a cell on the terrain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellContent {
    /// Terrain only.
    Empty,
    /// There is an object on the terrain.
    Object(usize), // index in the world's object catalog
    /// There is a creature on the terrain.
    Creature(CreatureId),
}

/// State of a creature's turn cycle (pattern State).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureState {
    /// The creature is awake and executes its program normally.
    Awake,
    /// The creature is asleep for `n` more ticks; it does not execute.
    Sleeping(u32),
}

/// A living creature placed on the map.
///
/// `species` is a shared reference (`Rc<Species>`) to the world's catalog
/// (Flyweight pattern): two creatures of the same species point to the same
/// instance, and cloning a creature clones only the `Rc`, never the `Species`.
#[derive(Debug, Clone)]
pub struct Creature {
    pub id: CreatureId,
    pub species: Rc<Species>,
    pub x: usize,
    pub y: usize,
    /// Current health.
    pub health: i64,
    /// Current age.
    pub age: i64,
    /// Maximum lifespan.
    pub lifespan: i64,
    /// Turn cycle state: Awake or Sleeping(n). Replaces the external
    /// `sleep_turns` counter; the state is managed internally.
    pub state: CreatureState,
    /// Next line index to execute at the start of the next turn.
    pub pc: usize,
    /// Variables owned by this creature, separate from other creatures.
    pub variables: HashMap<String, Value>,
}

/// Memento of a creature's mutable state at the start of a turn.
/// Captures a copy of the state so it can be restored or diagnosed after a
/// failed turn without leaving the world in a corrupt intermediate state.
#[derive(Debug, Clone)]
pub struct CreatureMemento {
    pub id: CreatureId,
    pub health: i64,
    pub age: i64,
    pub x: usize,
    pub y: usize,
    pub pc: usize,
    pub state: CreatureState,
    pub variables: HashMap<String, Value>,
}

impl Creature {
    /// Captures the current mutable state of this creature in a memento.
    pub fn memento(&self) -> CreatureMemento {
        CreatureMemento {
            id: self.id,
            health: self.health,
            age: self.age,
            x: self.x,
            y: self.y,
            pc: self.pc,
            state: self.state.clone(),
            variables: self.variables.clone(),
        }
    }
}
