//! `ActionContext`: the abstraction that actions depend on instead of the
//! concrete `World` type. `World` implements this trait, so actions are
//! decoupled from `World` (Dependency Inversion Principle). Tests can provide
//! a mock context to exercise actions in isolation.

use crate::diagnostic::Diagnostic;
use crate::event::Pos;
use crate::species::Species;
use crate::world::creature::{CellContent, Creature};
use crate::world::CreatureId;
use std::rc::Rc;

/// The world-view that actions depend on. `World` implements this trait.
pub trait ActionContext {
    // --- Read-only queries ---

    /// Position of the creature, or a diagnostic if unknown.
    fn creature_position(&self, id: CreatureId) -> Result<Pos, Diagnostic>;

    /// Species name of the creature, or a diagnostic if unknown.
    fn creature_species(&self, id: CreatureId) -> Result<String, Diagnostic>;

    /// The creature itself, or `None` if unknown.
    fn creature(&self, id: CreatureId) -> Option<&Creature>;

    /// Current tick counter.
    fn tick(&self) -> u64;

    /// Number of map columns.
    fn columns(&self) -> usize;

    /// Number of map rows.
    fn rows(&self) -> usize;

    /// True if cell `(x, y)` is inside the map and free (terrain only).
    fn cell_is_free(&self, x: usize, y: usize) -> bool;

    /// Content of cell `(x, y)` (cloned).
    fn content_at(&self, x: usize, y: usize) -> CellContent;

    /// Terrain catalog index of cell `(x, y)`.
    fn terrain_at(&self, x: usize, y: usize) -> usize;

    // --- Mutating operations ---

    /// Mutable reference to the creature, or `None` if unknown.
    fn creature_mut(&mut self, id: CreatureId) -> Option<&mut Creature>;

    /// Removes the creature, freeing its cell.
    fn remove_creature(&mut self, id: CreatureId) -> Result<(), Diagnostic>;

    /// Places a creature of the given species on a free cell.
    fn place_creature(
        &mut self,
        species: &Rc<Species>,
        x: usize,
        y: usize,
    ) -> Result<CreatureId, Diagnostic>;

    /// Sets the creature's health.
    fn set_creature_health(&mut self, id: CreatureId, health: i64) -> Result<(), Diagnostic>;

    /// Moves a creature to a new cell.
    fn move_creature_to(&mut self, id: CreatureId, x: usize, y: usize) -> Result<(), Diagnostic>;

    /// Registers a roar emitted by the creature.
    fn add_roar(&mut self, id: CreatureId, species: String, pos: Pos, text: String);

    /// Damages (or heals, if negative) the object or terrain content of a cell.
    fn damage_cell_content(
        &mut self,
        x: usize,
        y: usize,
        damage: i64,
    ) -> Result<(), Diagnostic>;

    /// Damages (or heals) the reserve of a terrain catalog entry.
    fn damage_terrain_reserve(
        &mut self,
        terrain_idx: usize,
        damage: i64,
    ) -> Result<(), Diagnostic>;
}
