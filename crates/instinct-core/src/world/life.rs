//! Creature lifecycle: placement, removal, access, and mutation.

use super::creature::{CellContent, Creature, CreatureId, CreatureMemento, CreatureState};
use super::World;
use crate::diagnostic::Diagnostic;
use crate::event::Pos;
use crate::species::Species;
use crate::value::Value;
use std::collections::HashMap;
use std::rc::Rc;

impl World {
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
}
