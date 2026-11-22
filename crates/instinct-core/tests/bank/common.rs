use instinct_core::resources::load_resources;
use instinct_core::sim::Simulation;
use instinct_core::world::{CreatureId, CreatureMemento, World};
use std::path::Path;
use std::rc::Rc;

/// Bank resources root directory: `examples/`, relative to this crate's
/// manifest.
pub fn examples_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
}

/// A simulation loaded from `examples/` with creatures placed by name.
pub struct BankWorld {
    pub sim: Simulation,
    /// Species name → creature id, for looking up mementos after ticks.
    pub creature_ids: Vec<(String, instinct_core::world::CreatureId)>,
}

impl BankWorld {
    /// Loads resources from `examples/`, builds the world from the named map,
    /// places the given creatures, and creates a seeded simulation.
    pub fn from_examples(map_name: &str, placements: &[(&str, usize, usize)]) -> Self {
        let dir = examples_dir();
        let root = Path::new(&dir);
        let resources = load_resources(root);
        assert!(
            resources.diagnostics.is_empty(),
            "load_resources diagnostics: {:?}",
            resources.diagnostics
        );

        let map = resources
            .maps
            .iter()
            .find(|m| m.name == map_name)
            .expect("map not found in examples/");

        let mut world = World::from_map(map, &resources.terrains, &resources.objects);
        assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);

        let mut creature_ids: Vec<(String, instinct_core::world::CreatureId)> = Vec::new();
        for (species_name, x, y) in placements {
            let species = resources
                .creatures
                .iter()
                .find(|s| s.name == *species_name)
                .expect("species not found in examples/creatures/");
            let species_rc = Rc::new(species.clone());
            let id = world
                .place_creature(&species_rc, *x, *y)
                .expect("place creature");
            creature_ids.push((species_name.to_string(), id));
        }

        let mut sim = Simulation::new(world);
        sim.set_rng_seed(1);
        BankWorld { sim, creature_ids }
    }

    /// Advances the simulation by one tick and returns the emitted events.
    pub fn step_tick(&mut self) -> Vec<instinct_core::event::Event> {
        self.sim.step_tick()
    }

    /// Advances the simulation by `n` ticks (events are discarded).
    pub fn step_ticks(&mut self, n: u64) {
        for _ in 0..n {
            self.sim.step_tick();
        }
    }

    /// Encuentra el id de una criatura viva en (x, y) que no esté en `exclude`.
    /// Lee directamente del almacén (el recién nacido no está en `creature_ids`).
    pub fn find_child_at(
        &self,
        x: usize,
        y: usize,
        exclude: &[instinct_core::world::CreatureId],
    ) -> Option<instinct_core::world::CreatureId> {
        self.sim
            .world
            .creatures
            .iter()
            .enumerate()
            .find_map(|(index, slot)| {
                let c = slot.as_ref()?;
                if exclude.contains(&CreatureId(index)) {
                    return None;
                }
                if c.x == x && c.y == y {
                    Some(CreatureId(index))
                } else {
                    None
                }
            })
    }

    /// Returns the memento of the creature with the given species name.
    pub fn memento(&self, species_name: &str) -> Option<CreatureMemento> {
        let id = self
            .creature_ids
            .iter()
            .find(|(name, _)| name == species_name)
            .map(|(_, id)| *id)?;
        let creature = self
            .sim
            .world
            .creatures
            .get(id.0)
            .and_then(Option::as_ref)?;
        Some(creature.memento())
    }
}
