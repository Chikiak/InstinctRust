//! World model: terrain grid, objects, creatures, ticks, and events.
//!
//! The world owns the seeded generator (with interior mutability confined to
//! the generator) and the shared catalog of actions and perceptions. Each
//! creature keeps its own variable map.

use crate::actions::{ActionResult, Catalog};
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::expr::EvalContext;
use crate::perception::CreatureView;
use crate::resources::{Map, Object, Terrain};
use crate::rng::Rng;
use crate::species::Species;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;

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

/// A living creature placed on the map.
#[derive(Debug, Clone)]
pub struct Creature {
    pub id: CreatureId,
    pub species: Species,
    pub x: usize,
    pub y: usize,
    /// Current health.
    pub health: i64,
    /// Current age.
    pub age: i64,
    /// Maximum lifespan.
    pub lifespan: i64,
    /// Turns of sleep remaining (from `wait`). Zero means awake.
    pub sleep_turns: i64,
    /// Next line index to execute at the start of the next turn.
    pub pc: usize,
    /// Variables owned by this creature, separate from other creatures.
    pub variables: HashMap<String, Value>,
}

/// The world: terrain grid, objects, creatures, tick counter, and event log.
#[derive(Debug, Clone)]
pub struct World {
    pub map_name: String,
    pub columns: usize,
    pub rows: usize,
    /// Terrain of each cell, indexed as [row][column].
    pub terrain: Vec<Vec<usize>>, // index in the terrain catalog
    /// Optional content (object or creature) of each cell.
    pub content: Vec<Vec<CellContent>>,
    /// Creatures by id. The position in this vec is not the id; id is stable.
    pub creatures: Vec<Creature>,
    /// Next creature id to assign.
    next_creature_id: usize,
    /// Number of completed ticks.
    pub ticks: u64,
    /// Events emitted since the last consumer drained them.
    pub events: Vec<Event>,
    pub diagnostics: Vec<Diagnostic>,
    /// Shared catalog of actions and perceptions.
    pub catalog: Catalog,
    /// Seeded pseudo-random generator. Interior mutability is confined to this
    /// field so `random` can advance it through an immutable world reference.
    pub rng: RefCell<Rng>,
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
                content: Vec::new(),
                creatures: Vec::new(),
                next_creature_id: 0,
                ticks: 0,
                events: Vec::new(),
                diagnostics,
                catalog: Catalog::default(),
                rng: RefCell::new(Rng::new(0)),
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
            content,
            creatures: Vec::new(),
            next_creature_id: 0,
            ticks: 0,
            events: Vec::new(),
            diagnostics,
            catalog: Catalog::default(),
            rng: RefCell::new(Rng::new(0)),
        }
    }

    // -----------------------------------------------------------------------
    // Creatures: placement, removal, accessors.
    // -----------------------------------------------------------------------

    /// Places a creature on a free cell. Returns the assigned `CreatureId` or
    /// a `Diagnostic` naming the coordinates when the cell is out of bounds or
    /// already occupied. The world is unchanged on failure.
    pub fn place_creature(
        &mut self,
        species: Species,
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

        let id = CreatureId(self.next_creature_id);
        self.next_creature_id += 1;
        let health = species.health;
        let lifespan = species.lifespan;
        let creature = Creature {
            id,
            species,
            x,
            y,
            health,
            age: 0,
            lifespan,
            sleep_turns: 0,
            pc: 0,
            variables: HashMap::new(),
        };
        self.content[y][x] = CellContent::Creature(id);
        self.creatures.push(creature);
        Ok(id)
    }

    /// Removes the creature with the given id, freeing its cell. Returns a
    /// diagnostic if the id does not exist.
    pub fn remove_creature(&mut self, creature_id: CreatureId) -> Result<(), Diagnostic> {
        let pos = match self.creature_position(creature_id) {
            Ok(p) => p,
            Err(_) => {
                return Err(Diagnostic::new(
                    "world",
                    0,
                    format!("unknown creature id {creature_id:?}"),
                ));
            }
        };
        self.content[pos.y][pos.x] = CellContent::Empty;
        self.creatures.retain(|c| c.id != creature_id);
        Ok(())
    }

    /// Returns the position of the creature, or a diagnostic if unknown.
    pub fn creature_position(&self, creature_id: CreatureId) -> Result<Pos, Diagnostic> {
        self.creatures
            .iter()
            .find(|c| c.id == creature_id)
            .map(|c| Pos { x: c.x, y: c.y })
            .ok_or_else(|| {
                Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
            })
    }

    /// Returns the species name of the creature, or a diagnostic if unknown.
    pub fn creature_species(&self, creature_id: CreatureId) -> Result<String, Diagnostic> {
        self.creatures
            .iter()
            .find(|c| c.id == creature_id)
            .map(|c| c.species.name.clone())
            .ok_or_else(|| {
                Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
            })
    }

    /// Returns the creature with the given id, or `None`.
    fn creature(&self, id: CreatureId) -> Option<&Creature> {
        self.creatures.iter().find(|c| c.id == id)
    }

    /// Returns the creature with the given id mutably, or `None`.
    fn creature_mut(&mut self, id: CreatureId) -> Option<&mut Creature> {
        self.creatures.iter_mut().find(|c| c.id == id)
    }

    /// Returns the value of a creature variable, or `None` if unassigned or
    /// unknown creature.
    pub fn variable(&self, creature_id: CreatureId, name: &str) -> Option<Value> {
        self.creature(creature_id)
            .and_then(|c| c.variables.get(name).cloned())
    }

    /// Assigns a variable on a creature. Rejects any name the catalog knows
    /// as an action or a perception.
    pub fn set_variable(
        &mut self,
        creature_id: CreatureId,
        name: &str,
        value: Value,
    ) -> Result<(), Diagnostic> {
        if self.catalog.is_action_name(name) || self.catalog.is_perception_name(name) {
            return Err(Diagnostic::new(
                "world",
                1,
                format!("cannot assign to '{name}': it is a reserved action or perception name"),
            ));
        }
        let creature = self.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 1, format!("unknown creature id {creature_id:?}"))
        })?;
        creature.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Seeds the world's generator. Two worlds with the same seed produce the
    /// same `random` sequence and turn order.
    pub fn set_rng_seed(&mut self, seed: u64) {
        self.rng.replace(Rng::new(seed));
    }

    /// Returns the current tick count.
    pub fn tick(&self) -> u64 {
        self.ticks
    }

    /// Advances the tick counter by one.
    pub fn advance_tick(&mut self) {
        self.ticks += 1;
    }

    /// Appends an event to the world's event log.
    pub fn emit_event(&mut self, event: Event) {
        self.events.push(event);
    }

    /// Drains all pending events, returning them and leaving the log empty.
    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Returns true if the cell at (x, y) is inside the map and has no object
    /// and no creature (terrain only).
    pub fn cell_is_free(&self, x: usize, y: usize) -> bool {
        if x >= self.columns || y >= self.rows {
            return false;
        }
        matches!(self.content[y][x], CellContent::Empty)
    }

    /// Moves a creature to a new cell. The caller must have verified the cell
    /// is free and in bounds. Updates both the creature and the grid.
    pub fn move_creature_to(
        &mut self,
        creature_id: CreatureId,
        x: usize,
        y: usize,
    ) -> Result<(), Diagnostic> {
        let old = self.creature_position(creature_id)?;
        self.content[old.y][old.x] = CellContent::Empty;
        for creature in self.creatures.iter_mut() {
            if creature.id == creature_id {
                creature.x = x;
                creature.y = y;
                break;
            }
        }
        self.content[y][x] = CellContent::Creature(creature_id);
        Ok(())
    }

    /// Applies the result of an action: sets sleep turns and appends events.
    pub fn apply_action_result(
        &mut self,
        creature_id: CreatureId,
        result: ActionResult,
    ) -> Result<(), Diagnostic> {
        if let Some(creature) = self.creatures.iter_mut().find(|c| c.id == creature_id) {
            creature.sleep_turns = result.sleep_turns;
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

    // -----------------------------------------------------------------------
    // End of tick laws.
    // -----------------------------------------------------------------------

    /// Applies the end-of-tick laws to every creature: -1 health, +1 age, and
    /// death when health <= 0 or age >= lifespan. Returns `Died` events for
    /// creatures that die and removes them from the map.
    pub fn end_of_tick_creatures(&mut self) -> Vec<Event> {
        let mut died: Vec<Event> = Vec::new();
        let mut dead_ids: Vec<CreatureId> = Vec::new();

        for creature in self.creatures.iter() {
            let next_health = creature.health - 1;
            let next_age = creature.age + 1;
            let dead = next_health <= 0 || next_age >= creature.lifespan;
            if dead {
                dead_ids.push(creature.id);
                let cause = if next_health <= 0 { "vida" } else { "edad" };
                died.push(Event::Died {
                    tick: self.ticks,
                    species: creature.species.name.clone(),
                    pos: Pos {
                        x: creature.x,
                        y: creature.y,
                    },
                    cause: cause.to_string(),
                });
            }
        }

        // Apply health/age changes (to survivors) and remove the dead.
        for creature in self.creatures.iter_mut() {
            if !dead_ids.contains(&creature.id) {
                creature.health -= 1;
                creature.age += 1;
            }
        }
        for id in &dead_ids {
            let _ = self.remove_creature(*id);
        }
        died
    }

    /// Regenerates terrain reserves up to `resource_max` and removes objects
    /// whose reserve is 0 or less, freeing their cells.
    pub fn end_of_tick_terrain_and_objects(
        &mut self,
        terrains: &mut [Terrain],
        objects: &[Object],
    ) -> Vec<Pos> {
        let mut freed: Vec<Pos> = Vec::new();

        // Regenerate each terrain once per tick (not once per cell).
        for terrain in terrains.iter_mut() {
            if terrain.reserve < terrain.resource_max {
                let new_reserve = terrain.reserve + terrain.regen;
                terrain.reserve = new_reserve.min(terrain.resource_max);
            }
        }

        // Remove exhausted objects and free their cells.
        for y in 0..self.rows {
            for x in 0..self.columns {
                if let CellContent::Object(obj_idx) = &self.content[y][x] {
                    if objects[*obj_idx].reserve <= 0 {
                        self.content[y][x] = CellContent::Empty;
                        freed.push(Pos { x, y });
                    }
                }
            }
        }
        freed
    }

    // -----------------------------------------------------------------------
    // Drawing.
    // -----------------------------------------------------------------------

    /// Draws the world in ASCII: the creature (first letter of its species)
    /// if there is one, otherwise the object's character if there is one,
    /// otherwise the terrain's character.
    pub fn draw(&self, terrains: &[Terrain], objects: &[Object]) -> String {
        let mut output = String::new();
        for row in 0..self.rows {
            for column in 0..self.columns {
                let c = match &self.content[row][column] {
                    CellContent::Creature(id) => {
                        let creature = self
                            .creatures
                            .iter()
                            .find(|c| c.id == *id)
                            .expect("grid references a missing creature");
                        creature.species.name.chars().next().unwrap_or('?')
                    }
                    CellContent::Object(idx) => objects[*idx].character,
                    CellContent::Empty => terrains[self.terrain[row][column]].character,
                };
                output.push(c);
            }
            output.push('\n');
        }
        output
    }
}

/// Read-only view of a creature and its world, used to evaluate perceptions.
/// Nothing here mutates the world except `random`, which advances the
/// generator through interior mutability.
#[derive(Debug)]
struct CreatureSnapshot<'a> {
    world: &'a World,
    creature_id: CreatureId,
}

impl CreatureView for CreatureSnapshot<'_> {
    fn health(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.health)
            .unwrap_or(0)
    }

    fn age(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.age)
            .unwrap_or(0)
    }

    fn lifespan(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.lifespan)
            .unwrap_or(0)
    }

    fn x(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.x as i64)
            .unwrap_or(0)
    }

    fn y(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.y as i64)
            .unwrap_or(0)
    }

    fn tick(&self) -> u64 {
        self.world.ticks
    }

    fn random(&self) -> i64 {
        self.world.rng.borrow_mut().next_bounded(99)
    }
}

/// Evaluation context for a single creature. Resolves perceptions first, then
/// variables. Execution reads the world immutably; the world applies actions
/// afterward.
pub struct CreatureContext<'a> {
    world: &'a World,
    creature_id: CreatureId,
}

impl<'a> CreatureContext<'a> {
    pub fn new(world: &'a World, creature_id: CreatureId) -> Self {
        CreatureContext { world, creature_id }
    }
}

impl<'a> EvalContext for CreatureContext<'a> {
    fn resolve(&self, name: &str) -> Option<Value> {
        // Perceptions take priority over variables.
        if let Some(perception) = self.world.catalog.resolve_perception(name) {
            let view = CreatureSnapshot {
                world: self.world,
                creature_id: self.creature_id,
            };
            return Some(perception.read(&view));
        }
        self.world.variable(self.creature_id, name)
    }
}
