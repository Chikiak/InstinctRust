//! `ActionContext`: the abstraction that actions depend on instead of the
//! concrete `World` type. `World` implements this trait, so actions are
//! decoupled from `World` (Dependency Inversion Principle). Tests can provide
//! a mock context to exercise actions in isolation.
//!
//! The trait is split into `ReadContext` (read-only queries) and
//! `WriteContext` (mutating operations). `ActionContext` is a supertrait
//! requiring both (Interface Segregation Principle).

use crate::diagnostic::Diagnostic;
use crate::event::Pos;
use crate::species::Species;
use crate::world::creature::{CellContent, Creature};
use crate::world::CreatureId;
use std::rc::Rc;

/// Read-only view of the world that actions depend on.
pub trait ReadContext {
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
}

/// Mutating operations on the world that actions depend on.
pub trait WriteContext {
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
    fn damage_cell_content(&mut self, x: usize, y: usize, damage: i64) -> Result<(), Diagnostic>;

    /// Damages (or heals) the reserve of a terrain catalog entry.
    fn damage_terrain_reserve(&mut self, terrain_idx: usize, damage: i64)
        -> Result<(), Diagnostic>;
}

/// The world-view that actions depend on. `World` implements this trait.
/// Requires both `ReadContext` and `WriteContext` (supertrait, no methods of
/// its own).
pub trait ActionContext: ReadContext + WriteContext {}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time check: a type can implement only ReadContext.
    #[allow(dead_code)]
    struct ReadOnly;
    impl ReadContext for ReadOnly {
        fn creature_position(&self, _id: CreatureId) -> Result<Pos, Diagnostic> {
            unimplemented!()
        }
        fn creature_species(&self, _id: CreatureId) -> Result<String, Diagnostic> {
            unimplemented!()
        }
        fn creature(&self, _id: CreatureId) -> Option<&Creature> {
            unimplemented!()
        }
        fn tick(&self) -> u64 {
            unimplemented!()
        }
        fn columns(&self) -> usize {
            unimplemented!()
        }
        fn rows(&self) -> usize {
            unimplemented!()
        }
        fn cell_is_free(&self, _x: usize, _y: usize) -> bool {
            unimplemented!()
        }
        fn content_at(&self, _x: usize, _y: usize) -> CellContent {
            unimplemented!()
        }
        fn terrain_at(&self, _x: usize, _y: usize) -> usize {
            unimplemented!()
        }
    }

    // Compile-time check: a type can implement only WriteContext.
    #[allow(dead_code)]
    struct WriteOnly;
    impl WriteContext for WriteOnly {
        fn creature_mut(&mut self, _id: CreatureId) -> Option<&mut Creature> {
            unimplemented!()
        }
        fn remove_creature(&mut self, _id: CreatureId) -> Result<(), Diagnostic> {
            unimplemented!()
        }
        fn place_creature(
            &mut self,
            _species: &Rc<Species>,
            _x: usize,
            _y: usize,
        ) -> Result<CreatureId, Diagnostic> {
            unimplemented!()
        }
        fn set_creature_health(&mut self, _id: CreatureId, _health: i64) -> Result<(), Diagnostic> {
            unimplemented!()
        }
        fn move_creature_to(
            &mut self,
            _id: CreatureId,
            _x: usize,
            _y: usize,
        ) -> Result<(), Diagnostic> {
            unimplemented!()
        }
        fn add_roar(&mut self, _id: CreatureId, _species: String, _pos: Pos, _text: String) {}
        fn damage_cell_content(
            &mut self,
            _x: usize,
            _y: usize,
            _damage: i64,
        ) -> Result<(), Diagnostic> {
            unimplemented!()
        }
        fn damage_terrain_reserve(
            &mut self,
            _terrain_idx: usize,
            _damage: i64,
        ) -> Result<(), Diagnostic> {
            unimplemented!()
        }
    }

    #[test]
    fn read_context_is_object_safe() {
        fn _assert_read_context(_ctx: &dyn ReadContext) {}
    }

    #[test]
    fn write_context_is_object_safe() {
        fn _assert_write_context(_ctx: &mut dyn WriteContext) {}
    }
}
