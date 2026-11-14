//! Species definition: a kind of creature with stable identity.
//! Since F0-06 a species carries its compiled program.
//!
//! Flyweight (F4-05): `Species` is intrinsic, immutable data once registered
//! (no setters); `program` is shared via `Rc`. Creatures point at the species
//! with `Rc<Species>` (chosen over `SpeciesId` because the shared program
//! travels by reference), so cloning a creature clones only the `Rc`, never
//! the `Species`.

use crate::program::Program;
use std::rc::Rc;

/// A kind of creature. Stores static traits and its compiled program.
///
/// Immutable after creation: its fields are read-only and there are no
/// setters. Shares its compiled program via `Rc<Program>`.
#[derive(Debug, Clone)]
pub struct Species {
    pub name: String,
    pub faction: String,
    pub health: i64,
    pub vision: i64,
    pub lifespan: i64,
    /// The compiled program, if the creature file compiled successfully.
    pub program: Option<Rc<Program>>,
}

impl Species {
    /// Creates a species with only a name (default attribute values).
    pub fn new(name: impl Into<String>) -> Self {
        Species {
            name: name.into(),
            faction: String::new(),
            health: 0,
            vision: 0,
            lifespan: 0,
            program: None,
        }
    }

    /// Creates a species with all attributes and a compiled program.
    pub fn with_program(
        name: impl Into<String>,
        faction: impl Into<String>,
        health: i64,
        vision: i64,
        lifespan: i64,
        program: Rc<Program>,
    ) -> Self {
        Species {
            name: name.into(),
            faction: faction.into(),
            health,
            vision,
            lifespan,
            program: Some(program),
        }
    }
}

impl PartialEq for Species {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.faction == other.faction
            && self.health == other.health
            && self.vision == other.vision
            && self.lifespan == other.lifespan
    }
}

impl Eq for Species {}
