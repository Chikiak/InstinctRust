//! Read-only structured snapshot of the world for rendering (pre-UI).
//!
//! `World::snapshot(&self)` builds an immutable DTO from public world fields
//! without mutating anything: no RNG, no `println!`, no floats, no external
//! dependencies. The DTO is `Clone`, `Debug`, `PartialEq`, and `Eq` so tests
//! and a future GUI can inspect, compare, and print it freely.

use super::creature::{CellContent, CreatureId, CreatureState};
use super::World;
use crate::event::Event;

/// Immutable snapshot of the whole world, safe to hand to a GUI renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub columns: usize,
    pub rows: usize,
    pub ticks: u64,
    pub map_name: String,
    pub cells: Vec<Vec<CellSnapshot>>,
    pub creatures: Vec<LiveCreature>,
    pub objects: Vec<ObjectInfo>,
    pub terrains: Vec<TerrainInfo>,
    pub events: Vec<Event>,
}

/// Read-only description of a single cell's visible content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellSnapshot {
    pub terrain_char: char,
    pub object_char: Option<char>,
    pub creature: Option<CreatureId>,
}

/// Read-only description of a living creature.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveCreature {
    pub id: CreatureId,
    pub species_name: String,
    pub faction: String,
    pub x: usize,
    pub y: usize,
    pub health: i64,
    pub age: i64,
    pub lifespan: i64,
    pub state: CreatureState,
    pub pc: usize,
}

/// Read-only description of an object in the world catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectInfo {
    pub index: usize,
    pub name: String,
    pub char: char,
    pub reserve: i64,
}

/// Read-only description of a terrain in the world catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrainInfo {
    pub name: String,
    pub char: char,
    pub reserve: i64,
    pub resource_max: i64,
}

impl World {
    /// Builds a read-only snapshot of the world without mutating anything.
    ///
    /// The snapshot clones the data a renderer needs (cells, creatures,
    /// objects, terrains, events) so the GUI can never corrupt the kernel.
    pub fn snapshot(&self) -> WorldSnapshot {
        let cells: Vec<Vec<CellSnapshot>> = (0..self.rows)
            .map(|y| {
                (0..self.columns)
                    .map(|x| {
                        let terrain_char = self.terrains[self.terrain[y][x]].character;
                        let (object_char, creature) = match &self.content[y][x] {
                            CellContent::Object(idx) => (Some(self.objects[*idx].character), None),
                            CellContent::Creature(id) => (None, Some(*id)),
                            CellContent::Empty => (None, None),
                        };
                        CellSnapshot {
                            terrain_char,
                            object_char,
                            creature,
                        }
                    })
                    .collect()
            })
            .collect();

        let creatures: Vec<LiveCreature> = self
            .creatures
            .iter()
            .flatten()
            .map(|creature| LiveCreature {
                id: creature.id,
                species_name: creature.species.name.clone(),
                faction: creature.species.faction.clone(),
                x: creature.x,
                y: creature.y,
                health: creature.health,
                age: creature.age,
                lifespan: creature.lifespan,
                state: creature.state.clone(),
                pc: creature.pc,
            })
            .collect();

        let objects: Vec<ObjectInfo> = self
            .objects
            .iter()
            .enumerate()
            .map(|(index, object)| ObjectInfo {
                index,
                name: object.name.clone(),
                char: object.character,
                reserve: object.reserve,
            })
            .collect();

        let terrains: Vec<TerrainInfo> = self
            .terrains
            .iter()
            .map(|terrain| TerrainInfo {
                name: terrain.name.clone(),
                char: terrain.character,
                reserve: terrain.reserve,
                resource_max: terrain.resource_max,
            })
            .collect();

        WorldSnapshot {
            columns: self.columns,
            rows: self.rows,
            ticks: self.ticks,
            map_name: self.map_name.clone(),
            cells,
            creatures,
            objects,
            terrains,
            events: self.events.clone(),
        }
    }
}
