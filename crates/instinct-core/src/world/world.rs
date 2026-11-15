//! World structure and `from_map` constructor.

use super::creature::{CellContent, Creature};
use super::roars::Roar;
use crate::actions::Catalog;
use crate::diagnostic::Diagnostic;
use crate::event::{Event, WorldOutput};
use crate::resources::{Map, Object, Terrain};
use crate::rng::{Rng, RngSource};
use crate::species::Species;
use std::cell::RefCell;
use std::rc::Rc;

/// The world: terrain grid, objects, creatures, tick counter, and event log.
///
/// `output` (output port) is excluded from `Clone` and omitted from `Debug`
/// because `Box<dyn WorldOutput>` does not implement those traits.
pub struct World {
    pub map_name: String,
    pub columns: usize,
    pub rows: usize,
    /// Terrain of each cell, indexed as [row][column].
    pub terrain: Vec<Vec<usize>>, // index in the terrain catalog
    /// Terrain catalog (reserves are mutated here when a cell's terrain is hit).
    pub terrains: Vec<Terrain>,
    /// Object catalog (reserves are mutated here when a cell's object is hit).
    pub objects: Vec<Object>,
    /// Optional content (object or creature) of each cell.
    pub content: Vec<Vec<CellContent>>,
    /// Species catalog owned by the world (Flyweight). Each entry is a shared
    /// `Rc<Species>`; creatures keep cheap clones of the same `Rc` without ever
    /// cloning the full `Species`.
    pub species: Vec<Rc<Species>>,
    /// Creatures by id: `creatures[id.0]` is `Some(creature)` while the
    /// creature lives and `None` is a free slot ready for reuse. The index is
    /// the stable `CreatureId`, so `creature(id)`/`creature_mut(id)` are O(1)
    /// with no linear search.
    pub creatures: Vec<Option<Creature>>,
    /// Free-list of creature ids whose slots can be reused. `place_creature`
    /// pops one here before growing `creatures`; when empty the next id is
    /// `creatures.len()`. Replaces the old `next_creature_id` counter; ids are
    /// stable within a simulation and never invalidated for other creatures.
    pub free_ids: Vec<usize>,
    /// Number of completed ticks.
    pub ticks: u64,
    /// Events emitted since the last consumer drained them.
    pub events: Vec<Event>,
    pub diagnostics: Vec<Diagnostic>,
    /// Shared catalog of actions and perceptions.
    pub catalog: Catalog,
    /// Seeded pseudo-random generator wrapped as a `RngSource` strategy object.
    /// Interior mutability is confined to this field so `random` can advance it
    /// through an immutable world reference.
    pub rng: RefCell<Box<dyn RngSource>>,
    /// Roars emitted in the current tick, associated with their tick and position.
    /// At the end of each tick (`sim.rs`) they are rotated to `roars_previous`.
    pub roars_current: Vec<Roar>,
    /// Roars from the previous tick; readable during the current tick (prepares
    /// `roars_near`/`last_roar` of F2).
    pub roars_previous: Vec<Roar>,
    /// Optional output port. Events emitted by `emit_event` are forwarded here
    /// in addition to being stored in `self.events`. `None` by default: the
    /// world works the same as before without an event consumer.
    pub output: Option<Box<dyn WorldOutput>>,
}

impl World {
    /// Builds a world from a loaded map and the catalogs.
    /// Characters that do not match any terrain or object produce a diagnostic;
    /// the terrain below a cell with an object is the first loaded terrain.
    pub fn from_map(map: &Map, terrains: &[Terrain], objects: &[Object]) -> Self {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        if terrains.is_empty() {
            diagnostics.push(Diagnostic::new(
                map.name.as_str(),
                1,
                "no terrains loaded to draw the map".to_string(),
            ));
            return World {
                map_name: map.name.clone(),
                columns: 0,
                rows: 0,
                terrain: Vec::new(),
                terrains: terrains.to_vec(),
                objects: objects.to_vec(),
                content: Vec::new(),
                species: Vec::new(),
                creatures: Vec::new(),
                free_ids: Vec::new(),
                ticks: 0,
                events: Vec::new(),
                diagnostics,
                catalog: Catalog::default(),
                rng: RefCell::new(Box::new(Rng::new(0))),
                roars_current: Vec::new(),
                roars_previous: Vec::new(),
                output: None,
            };
        }

        let first_terrain = 0usize;
        let columns = map.rows[0].len();
        let rows = map.rows.len();

        let mut terrain: Vec<Vec<usize>> = Vec::with_capacity(rows);
        let mut content: Vec<Vec<CellContent>> = Vec::with_capacity(rows);

        for (row_idx, row) in map.rows.iter().enumerate() {
            let mut row_terrain: Vec<usize> = Vec::with_capacity(columns);
            let mut row_content: Vec<CellContent> = Vec::with_capacity(columns);
            for c in row.iter() {
                // Look for the character among objects first, then terrains.
                let object_idx = objects.iter().position(|o| o.character == *c);
                if let Some(idx) = object_idx {
                    row_terrain.push(first_terrain);
                    row_content.push(CellContent::Object(idx));
                    continue;
                }
                let terrain_idx = terrains.iter().position(|t| t.character == *c);
                match terrain_idx {
                    Some(idx) => {
                        row_terrain.push(idx);
                        row_content.push(CellContent::Empty);
                    }
                    None => {
                        diagnostics.push(Diagnostic::new(
                            map.name.as_str(),
                            row_idx + 2,
                            format!("unknown character in the map: '{c}'"),
                        ));
                        row_terrain.push(first_terrain);
                        row_content.push(CellContent::Empty);
                    }
                }
            }
            terrain.push(row_terrain);
            content.push(row_content);
        }

        World {
            map_name: map.name.clone(),
            columns,
            rows,
            terrain,
            terrains: terrains.to_vec(),
            objects: objects.to_vec(),
            content,
            species: Vec::new(),
            creatures: Vec::new(),
            free_ids: Vec::new(),
            ticks: 0,
            events: Vec::new(),
            diagnostics,
            catalog: Catalog::default(),
            rng: RefCell::new(Box::new(Rng::new(0))),
            roars_current: Vec::new(),
            roars_previous: Vec::new(),
            output: None,
        }
    }
}
