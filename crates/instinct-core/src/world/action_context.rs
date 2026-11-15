//! `ActionContext` implementation for `World`.

use super::World;
use crate::actions::ActionContext;
use crate::diagnostic::Diagnostic;
use crate::event::Pos;
use crate::species::Species;
use crate::world::creature::{CellContent, CreatureId};
use crate::world::Creature;
use std::rc::Rc;

impl World {
    /// Number of map columns.
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Number of map rows.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Content of cell `(x, y)`, cloned. Returns `Empty` if out of bounds.
    pub fn content_at(&self, x: usize, y: usize) -> CellContent {
        self.content
            .get(y)
            .and_then(|row| row.get(x))
            .cloned()
            .unwrap_or(CellContent::Empty)
    }

    /// Terrain catalog index of cell `(x, y)`. Returns `0` if out of bounds.
    pub fn terrain_at(&self, x: usize, y: usize) -> usize {
        self.terrain
            .get(y)
            .and_then(|row| row.get(x))
            .copied()
            .unwrap_or(0)
    }

    /// Damages (or heals, if negative) the reserve of a terrain catalog entry.
    pub fn damage_terrain_reserve(
        &mut self,
        terrain_idx: usize,
        damage: i64,
    ) -> Result<(), Diagnostic> {
        let t = &mut self.terrains[terrain_idx];
        t.reserve = t.reserve.checked_sub(damage).ok_or_else(|| {
            Diagnostic::new("attack", 1, format!("overflow damaging terrain index {terrain_idx}"))
        })?;
        Ok(())
    }
}

impl ActionContext for World {
    fn creature_position(&self, id: CreatureId) -> Result<Pos, Diagnostic> {
        self.creature_position(id)
    }

    fn creature_species(&self, id: CreatureId) -> Result<String, Diagnostic> {
        self.creature_species(id)
    }

    fn creature(&self, id: CreatureId) -> Option<&Creature> {
        self.creature(id)
    }

    fn tick(&self) -> u64 {
        self.tick()
    }

    fn columns(&self) -> usize {
        self.columns()
    }

    fn rows(&self) -> usize {
        self.rows()
    }

    fn cell_is_free(&self, x: usize, y: usize) -> bool {
        self.cell_is_free(x, y)
    }

    fn content_at(&self, x: usize, y: usize) -> CellContent {
        self.content_at(x, y)
    }

    fn terrain_at(&self, x: usize, y: usize) -> usize {
        self.terrain_at(x, y)
    }

    fn creature_mut(&mut self, id: CreatureId) -> Option<&mut Creature> {
        self.creature_mut(id)
    }

    fn remove_creature(&mut self, id: CreatureId) -> Result<(), Diagnostic> {
        self.remove_creature(id)
    }

    fn place_creature(
        &mut self,
        species: &Rc<Species>,
        x: usize,
        y: usize,
    ) -> Result<CreatureId, Diagnostic> {
        self.place_creature(species, x, y)
    }

    fn set_creature_health(&mut self, id: CreatureId, health: i64) -> Result<(), Diagnostic> {
        self.set_creature_health(id, health)
    }

    fn move_creature_to(&mut self, id: CreatureId, x: usize, y: usize) -> Result<(), Diagnostic> {
        self.move_creature_to(id, x, y)
    }

    fn add_roar(&mut self, id: CreatureId, species: String, pos: Pos, text: String) {
        self.add_roar(id, species, pos, text)
    }

    fn damage_cell_content(
        &mut self,
        x: usize,
        y: usize,
        damage: i64,
    ) -> Result<(), Diagnostic> {
        self.damage_cell_content(x, y, damage)
    }

    fn damage_terrain_reserve(
        &mut self,
        terrain_idx: usize,
        damage: i64,
    ) -> Result<(), Diagnostic> {
        self.damage_terrain_reserve(terrain_idx, damage)
    }
}
