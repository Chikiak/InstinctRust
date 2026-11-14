//! End-of-tick laws: creature aging/damage, terrain regeneration, object
//! exhaustion, and cell damage.

use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use super::creature::{CellContent, CreatureId};
use super::World;

impl World {
    /// Damages (or heals, if `damage` is negative) the content of cell (x, y):
    /// reduces the reserve of the object or terrain at that cell using checked
    /// arithmetic. Out of bounds is a no-op without error. Creatures are not
    /// touched here (their health is managed by the caller). An object whose
    /// reserve reaches 0 or less is removed by the existing end-of-tick law.
    pub fn damage_cell_content(
        &mut self,
        x: usize,
        y: usize,
        damage: i64,
    ) -> Result<(), Diagnostic> {
        if x >= self.columns || y >= self.rows {
            return Ok(()); // out of bounds: no-op, no error
        }
        match &self.content[y][x] {
            CellContent::Object(obj_idx) => {
                let obj = &mut self.objects[*obj_idx];
                obj.reserve = obj.reserve.checked_sub(damage).ok_or_else(|| {
                    Diagnostic::new("world", 0, format!("overflow damaging object at ({x},{y})"))
                })?;
            }
            CellContent::Empty => {
                let t_idx = self.terrain[y][x];
                let t = &mut self.terrains[t_idx];
                t.reserve = t.reserve.checked_sub(damage).ok_or_else(|| {
                    Diagnostic::new(
                        "world",
                        0,
                        format!("overflow damaging terrain at ({x},{y})"),
                    )
                })?;
            }
            CellContent::Creature(_) => {
                // Creature health is managed by the caller (Attack).
            }
        }
        Ok(())
    }

    /// Applies the end-of-tick laws to every creature: -1 health, +1 age, and
    /// death when health <= 0 or age >= lifespan. Returns `Died` events for
    /// creatures that die and removes them from the map.
    ///
    /// Traverses the store by index (living ids), O(1) per access, no linear
    /// search and no `dead_ids.contains` in loops.
    ///
    /// Health and age are computed with checked arithmetic. If `health - 1` or
    /// `age + 1` overflows (e.g. health at `i64::MIN` or age at `i64::MAX`),
    /// it is treated as a runtime error: the creature dies with cause
    /// `"error"` without panicking in debug or release.
    pub fn end_of_tick_creatures(&mut self) -> Vec<Event> {
        let mut died: Vec<Event> = Vec::new();
        let mut dead_indices: Vec<usize> = Vec::new();
        // (index, next_health, next_age) of surviving creatures, to apply the
        // change after the scan without double indexing.
        let mut survivors: Vec<(usize, i64, i64)> = Vec::new();

        for (index, slot) in self.creatures.iter_mut().enumerate() {
            let Some(creature) = slot else { continue }; // free slot
            match (creature.health.checked_sub(1), creature.age.checked_add(1)) {
                (Some(next_health), Some(next_age)) => {
                    if next_health <= 0 || next_age >= creature.lifespan {
                        dead_indices.push(index);
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
                    } else {
                        survivors.push((index, next_health, next_age));
                    }
                }
                _ => {
                    // Overflow: runtime error, the creature dies.
                    dead_indices.push(index);
                    died.push(Event::Died {
                        tick: self.ticks,
                        species: creature.species.name.clone(),
                        pos: Pos {
                            x: creature.x,
                            y: creature.y,
                        },
                        cause: "error".to_string(),
                    });
                }
            }
        }

        // Apply health and age to survivors.
        for (index, next_health, next_age) in survivors {
            if let Some(creature) = self.creatures[index].as_mut() {
                creature.health = next_health;
                creature.age = next_age;
            }
        }
        // Remove the dead (by life, age, or runtime error), freeing their
        // slots without invalidating other ids.
        for index in dead_indices {
            let _ = self.remove_creature(CreatureId(index));
        }
        died
    }

    /// Regenerates terrain reserves up to `resource_max` and removes objects
    /// whose reserve is 0 or less, freeing their cells.
    ///
    /// Operates on the world's own catalogs (sole owner): it is the same
    /// reserve that `damage_cell_content` mutates, not a copy, so damage and
    /// regeneration never work on different reserves.
    pub fn end_of_tick_terrain_and_objects(&mut self) -> Vec<Pos> {
        let mut freed: Vec<Pos> = Vec::new();

        // Regenerate each terrain once per tick (not once per cell).
        for terrain in self.terrains.iter_mut() {
            if terrain.reserve < terrain.resource_max {
                let new_reserve = match terrain.reserve.checked_add(terrain.regen) {
                    Some(value) => value.min(terrain.resource_max),
                    None => terrain.resource_max,
                };
                terrain.reserve = new_reserve;
            }
        }

        // Remove exhausted objects and free their cells.
        for y in 0..self.rows {
            for x in 0..self.columns {
                if let CellContent::Object(obj_idx) = &self.content[y][x] {
                    if self.objects[*obj_idx].reserve <= 0 {
                        self.content[y][x] = CellContent::Empty;
                        freed.push(Pos { x, y });
                    }
                }
            }
        }
        freed
    }
}
