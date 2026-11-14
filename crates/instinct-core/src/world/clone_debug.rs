//! Clone and Debug implementations for World.

use crate::rng::Rng;
use std::cell::RefCell;
use super::creature::World;

impl Clone for World {
    fn clone(&self) -> Self {
        World {
            map_name: self.map_name.clone(),
            columns: self.columns,
            rows: self.rows,
            terrain: self.terrain.clone(),
            terrains: self.terrains.clone(),
            objects: self.objects.clone(),
            content: self.content.clone(),
            species: self.species.clone(),
            creatures: self.creatures.clone(),
            free_ids: self.free_ids.clone(),
            ticks: self.ticks,
            events: self.events.clone(),
            diagnostics: self.diagnostics.clone(),
            catalog: self.catalog.clone(),
            rng: RefCell::new(Box::new(Rng::new(0))), // cloning resets the RNG to seed 0
            roars_current: self.roars_current.clone(),
            roars_previous: self.roars_previous.clone(),
            // The output port is not cloned: the clone starts without a port.
            output: None,
        }
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("map_name", &self.map_name)
            .field("columns", &self.columns)
            .field("rows", &self.rows)
            .field("terrain", &self.terrain)
            .field("terrains", &self.terrains)
            .field("objects", &self.objects)
            .field("content", &self.content)
            .field("species", &self.species)
            .field("creatures", &self.creatures)
            .field("free_ids", &self.free_ids)
            .field("ticks", &self.ticks)
            .field("events", &self.events)
            .field("diagnostics", &self.diagnostics)
            .field("catalog", &self.catalog)
            .field("rng", &self.rng)
            .field("roars_current", &self.roars_current)
            .field("roars_previous", &self.roars_previous)
            .field("output", &self.output.is_some())
            .finish()
    }
}
