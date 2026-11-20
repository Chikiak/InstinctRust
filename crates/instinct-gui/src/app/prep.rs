//! Headless preparation state: placing and removing creatures over a loaded
//! map, building the recipe (map + seed + placements) that the GUI saves,
//! not the world. No eframe/egui knowledge: pure Rust, testable headless.

use crate::app::session::LoadedSession;
use instinct_core::diagnostic::Diagnostic;
use instinct_core::species::Species;
use instinct_core::world::World;
use std::rc::Rc;

/// Stable identifier of a placement in the recipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlacementId(pub usize);

/// One creature placement in the recipe.
#[derive(Debug, Clone)]
pub struct Placement {
    pub species: Rc<Species>,
    pub x: usize,
    pub y: usize,
    /// Stable creature id assigned by the world at placement time, used by
    /// `remove` to free the exact creature slot.
    pub creature_id: instinct_core::world::creature::CreatureId,
}

/// Recipe of a simulation: map name, seed, and placements.
#[derive(Debug, Clone)]
pub struct Recipe {
    pub map_name: String,
    pub seed: u64,
    pub placements: Vec<(String, usize, usize)>,
}

/// Headless preparation state.
pub struct PrepState {
    pub world: World,
    pub recipe: Recipe,
    pub next_placement_id: PlacementId,
    /// Maps each placement id to the index of its placement in
    /// `recipe.placements` (and in `placements`).
    pub id_to_index: Vec<usize>,
    /// Internal placements, indexed the same way as `recipe.placements`.
    /// Holds the `CreatureId` needed by `remove`.
    placements: Vec<Placement>,
}

impl PrepState {
    /// Enters preparation over `map_name` from `session`, building a world
    /// with `World::from_map` and leaving placements empty.
    pub fn enter_prep(
        session: &LoadedSession,
        map_name: &str,
        seed: u64,
    ) -> Result<PrepState, Diagnostic> {
        let map = session
            .maps()
            .iter()
            .find(|m| m.name == map_name)
            .ok_or_else(|| {
                Diagnostic::new("prep", 0, format!("map '{map_name}' not found in session"))
            })?;

        let world = World::from_map(
            map,
            &session.resources().terrains,
            &session.resources().objects,
        );

        Ok(PrepState {
            world,
            recipe: Recipe {
                map_name: map_name.to_string(),
                seed,
                placements: Vec::new(),
            },
            next_placement_id: PlacementId(0),
            id_to_index: Vec::new(),
            placements: Vec::new(),
        })
    }

    /// Places `species_name` at `(x, y)`, validating the species name against
    /// `session.species()` and delegating bounds / free-cell checks to the
    /// world. Multiple placements of the same species are allowed.
    pub fn place(
        &mut self,
        session: &LoadedSession,
        species_name: &str,
        x: usize,
        y: usize,
    ) -> Result<PlacementId, Diagnostic> {
        let species = session
            .species()
            .iter()
            .find(|s| s.name == species_name)
            .ok_or_else(|| {
                Diagnostic::new(
                    "prep",
                    0,
                    format!("species '{species_name}' not known in session"),
                )
            })?;

        let species = Rc::new(species.clone());
        let creature_id = self.world.place_creature(&species, x, y)?;

        let id = self.next_placement_id;
        let index = self.recipe.placements.len();
        self.recipe
            .placements
            .push((species_name.to_string(), x, y));
        self.placements.push(Placement {
            species,
            x,
            y,
            creature_id,
        });
        self.id_to_index.push(index);
        self.next_placement_id = PlacementId(id.0 + 1);

        Ok(id)
    }

    /// Removes the placement with `placement_id` from the world and from the
    /// recipe, freeing its cell again.
    pub fn remove(&mut self, placement_id: PlacementId) -> Result<(), Diagnostic> {
        let index = self
            .id_to_index
            .get(placement_id.0)
            .copied()
            .ok_or_else(|| {
                Diagnostic::new(
                    "prep",
                    0,
                    format!("placement id {placement_id:?} does not exist"),
                )
            })?;

        let creature_id = self.placements[index].creature_id;
        self.world.remove_creature(creature_id)?;

        // Remove from recipe and internal placements at `index`.
        self.recipe.placements.remove(index);
        self.placements.remove(index);

        // All recipe indices > `index` shift down by one.
        for slot in self.id_to_index.iter_mut() {
            if *slot > index {
                *slot -= 1;
            }
        }
        // Placement id `placement_id.0` no longer exists; remove its slot and
        // shift remaining placement ids down.
        self.id_to_index.remove(placement_id.0);

        Ok(())
    }

    /// True if cell `(x, y)` is occupied (object or creature).
    pub fn occupied_cell(&self, x: usize, y: usize) -> bool {
        !self.world.cell_is_free(x, y)
    }

    /// All placements as `(PlacementId, species_name, x, y)`.
    pub fn creatures_in_prep(&self) -> Vec<(PlacementId, String, usize, usize)> {
        self.recipe
            .placements
            .iter()
            .enumerate()
            .map(|(i, (name, x, y))| (PlacementId(i), name.clone(), *x, *y))
            .collect()
    }

    /// Number of placements in the recipe.
    pub fn placed_count(&self) -> usize {
        self.recipe.placements.len()
    }

    /// Returns the `PlacementId` of the creature at cell `(x, y)`, if any.
    pub fn placement_id_at(&self, x: usize, y: usize) -> Option<PlacementId> {
        let creature_id = self.world.creature_at(x, y)?;
        for (i, placement) in self.placements.iter().enumerate() {
            if placement.creature_id == creature_id {
                return Some(PlacementId(i));
            }
        }
        None
    }
}
