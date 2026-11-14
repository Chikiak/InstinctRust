//! Creature types and World core: creature placement, removal, accessors.

use crate::actions::{ActionResult, Catalog};
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos, WorldOutput};
use crate::resources::{Map, Object, Terrain};
use crate::rng::{Rng, RngSource};
use crate::species::Species;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use super::roars::Roar;

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

    // -----------------------------------------------------------------------
    // Creatures: placement, removal, accessors.
    // -----------------------------------------------------------------------

    /// Places a creature on a free cell. Returns the assigned `CreatureId` or
    /// a `Diagnostic` naming the coordinates when the cell is out of bounds or
    /// already occupied. The world is unchanged on failure.
    ///
    /// `species` is a reference to the shared catalog (Flyweight): the
    /// creature stores `Rc::clone(species)`, never a cloned `Species`, and the
    /// world ensures it has its own `Rc` in `species` (the owning catalog).
    pub fn place_creature(
        &mut self,
        species: &Rc<Species>,
        x: usize,
        y: usize,
    ) -> Result<CreatureId, Diagnostic> {
        if x >= self.columns || y >= self.rows {
            return Err(Diagnostic::new(
                "world",
                0,
                format!(
                    "cannot place creature at ({x},{y}): outside the map ({}x{})",
                    self.columns, self.rows
                ),
            ));
        }
        if !self.cell_is_free(x, y) {
            return Err(Diagnostic::new(
                "world",
                0,
                format!("cannot place creature at ({x},{y}): cell is occupied",),
            ));
        }

        // The world owns its copy of the `Rc<Species>` in the catalog;
        // if already registered (same pointer), the entry is reused.
        if !self.species.iter().any(|owned| Rc::ptr_eq(owned, species)) {
            self.species.push(Rc::clone(species));
        }

        // Reuse a free slot if one exists; otherwise grow the store. The index
        // is the stable id of the new creature.
        let id = match self.free_ids.pop() {
            Some(slot) => slot,
            None => {
                self.creatures.push(None);
                self.creatures.len() - 1
            }
        };
        let health = species.health;
        let lifespan = species.lifespan;
        let creature = Creature {
            id: CreatureId(id),
            species: Rc::clone(species), // flyweight: clone the Rc, not the Species
            x,
            y,
            health,
            age: 0,
            lifespan,
            state: CreatureState::Awake,
            pc: 0,
            variables: HashMap::new(),
        };
        self.content[y][x] = CellContent::Creature(CreatureId(id));
        self.creatures[id] = Some(creature);
        Ok(CreatureId(id))
    }

    /// Removes the creature with the given id, freeing its cell. Returns a
    /// diagnostic if the id does not exist.
    pub fn remove_creature(&mut self, creature_id: CreatureId) -> Result<(), Diagnostic> {
        let pos = self.creature_position(creature_id)?; // O(1): direct index
        self.content[pos.y][pos.x] = CellContent::Empty;
        // Free the slot without invalidating other creature ids.
        self.creatures[creature_id.0] = None;
        self.free_ids.push(creature_id.0);
        Ok(())
    }

    /// Ids of living creatures, in ascending id order (store order).
    /// `sim.step_tick` iterates over these ids; no linear search.
    pub fn live_creature_ids(&self) -> Vec<CreatureId> {
        self.creatures
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|_| CreatureId(index)))
            .collect()
    }

    /// Returns the position of the creature, or a diagnostic if unknown.
    /// O(1): indexes directly into `creatures[id.0]`.
    pub fn creature_position(&self, creature_id: CreatureId) -> Result<Pos, Diagnostic> {
        self.creature(creature_id)
            .map(|c| Pos { x: c.x, y: c.y })
            .ok_or_else(|| {
                Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
            })
    }

    /// Returns the species name of the creature, or a diagnostic if unknown.
    pub fn creature_species(&self, creature_id: CreatureId) -> Result<String, Diagnostic> {
        self.creature(creature_id)
            .map(|c| c.species.name.clone())
            .ok_or_else(|| {
                Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
            })
    }

    /// Returns the creature with the given id, or `None`. O(1): direct
    /// indexing; `None` both if the id is out of range or its slot is free.
    pub(crate) fn creature(&self, id: CreatureId) -> Option<&Creature> {
        self.creatures.get(id.0).and_then(Option::as_ref)
    }

    /// Returns the creature with the given id mutably, or `None`. O(1).
    pub(crate) fn creature_mut(&mut self, id: CreatureId) -> Option<&mut Creature> {
        self.creatures.get_mut(id.0).and_then(Option::as_mut)
    }

    /// Returns the value of a creature variable, or `None` if unassigned or
    /// unknown creature.
    pub fn variable(&self, creature_id: CreatureId, name: &str) -> Option<Value> {
        self.creature(creature_id)
            .and_then(|c| c.variables.get(name).cloned())
    }

    /// Sets the health of the creature with the given id. Returns a diagnostic
    /// if the id does not exist; never indexes out of range. `reproduce` uses
    /// this to adjust the offspring's health after `place_creature` (which uses
    /// `species.health`) to the value indicated in the action.
    pub fn set_creature_health(
        &mut self,
        creature_id: CreatureId,
        health: i64,
    ) -> Result<(), Diagnostic> {
        let creature = self.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        creature.health = health;
        Ok(())
    }

    /// Assigns a variable on a creature. Rejects any name the catalog knows
    /// as an action, a perception, a function, or a constant.
    pub fn set_variable(
        &mut self,
        creature_id: CreatureId,
        name: &str,
        value: Value,
    ) -> Result<(), Diagnostic> {
        if self.catalog.is_action_name(name)
            || self.catalog.is_perception_name(name)
            || self.catalog.is_function_name(name)
            || self.catalog.is_constant_name(name)
        {
            return Err(Diagnostic::new(
                "world",
                1,
                format!("cannot assign to '{name}': it is a reserved name"),
            ));
        }
        let creature = self.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 1, format!("unknown creature id {creature_id:?}"))
        })?;
        creature.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Restores a creature's state from a memento captured at the start of a
    /// turn. Returns `false` if the creature no longer exists (does nothing).
    /// Used to undo partial changes when a turn fails with a runtime error.
    pub fn restore_creature_from_memento(&mut self, memento: &CreatureMemento) -> bool {
        if let Some(creature) = self.creature_mut(memento.id) {
            creature.health = memento.health;
            creature.age = memento.age;
            creature.x = memento.x;
            creature.y = memento.y;
            creature.pc = memento.pc;
            creature.state = memento.state.clone();
            creature.variables = memento.variables.clone();
            true
        } else {
            false
        }
    }

    /// Seeds the world's generator. Two worlds with the same seed produce the
    /// same `random` sequence and turn order.
    pub fn set_rng_seed(&mut self, seed: u64) {
        *self.rng.borrow_mut() = Box::new(Rng::new(seed));
    }

    /// Injects a `RngSource` strategy (e.g. a deterministic `TestRng` for
    /// tests). Replaces the world's current generator.
    pub fn set_rng_source(&mut self, rng: Box<dyn RngSource>) {
        *self.rng.borrow_mut() = rng;
    }

    /// Connects an output port to the world. Events emitted by `emit_event`
    /// are forwarded to the port in addition to being stored in `self.events`.
    /// The port is optional: if nothing is connected, the world works as
    /// before. A port error never propagates to the tick.
    pub fn set_output(&mut self, output: Box<dyn WorldOutput>) {
        self.output = Some(output);
    }

    /// Returns the current tick count.
    pub fn tick(&self) -> u64 {
        self.ticks
    }

    /// Advances the tick counter by one.
    pub fn advance_tick(&mut self) {
        self.ticks += 1;
    }

    /// Appends an event to the world's event log and forwards it to the
    /// output port if one is connected. A failure in the port never aborts
    /// the tick.
    pub fn emit_event(&mut self, event: Event) {
        // Forward to the port before storing: the port borrows a reference to
        // the event, then the event is moved into the log.
        if let Some(output) = self.output.as_mut() {
            output.emit(&event);
        }
        self.events.push(event);
    }

    /// Drains all pending events, returning them and leaving the log empty.
    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Moves a creature to a new cell. The caller must have verified the cell
    /// is free and in bounds. Updates both the creature and the grid.
    pub fn move_creature_to(
        &mut self,
        creature_id: CreatureId,
        x: usize,
        y: usize,
    ) -> Result<(), Diagnostic> {
        if x >= self.columns || y >= self.rows {
            return Err(Diagnostic::new(
                "world",
                0,
                format!(
                    "cannot move creature {creature_id:?} to ({x},{y}): outside the map ({}x{})",
                    self.columns, self.rows
                ),
            ));
        }
        let old = self.creature_position(creature_id)?;
        self.content[old.y][old.x] = CellContent::Empty;
        if let Some(creature) = self.creature_mut(creature_id) {
            creature.x = x;
            creature.y = y;
        }
        self.content[y][x] = CellContent::Creature(creature_id);
        Ok(())
    }

    /// Applies the result of an action: sets the creature's turn state and
    /// appends events. `sleep_turns > 0` means `Sleeping(sleep_turns)`;
    /// zero means `Awake`.
    pub fn apply_action_result(
        &mut self,
        creature_id: CreatureId,
        result: ActionResult,
    ) -> Result<(), Diagnostic> {
        if let Some(creature) = self.creature_mut(creature_id) {
            creature.state = if result.sleep_turns > 0 {
                CreatureState::Sleeping(result.sleep_turns)
            } else {
                CreatureState::Awake
            };
        } else {
            return Err(Diagnostic::new(
                "world",
                0,
                format!("unknown creature id {creature_id:?}"),
            ));
        }
        for event in result.events {
            self.emit_event(event);
        }
        Ok(())
    }
}
