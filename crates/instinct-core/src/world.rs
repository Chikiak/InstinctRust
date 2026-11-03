//! World model: terrain grid, objects, creatures, ticks, and events.

use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::resources::{Map, Object, Terrain};
use crate::species::Species;

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
    /// Turns of sleep remaining (from `wait`). Zero means awake.
    pub sleep_turns: i64,
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
        let creature = Creature {
            id,
            species,
            x,
            y,
            sleep_turns: 0,
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
