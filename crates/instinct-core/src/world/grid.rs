//! Cell queries: terrain, object, creature, reserve, and free-cell checks.

use crate::perceptions::{Nearby, Nearest};
use super::creature::{CellContent, CreatureId};
use super::World;

impl World {
    /// Name of the terrain at cell `(x, y)`, or `None` if out of bounds or its
    /// index is not covered by the terrain catalog.
    pub fn terrain_name_at(&self, x: usize, y: usize) -> Option<&str> {
        self.terrain
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|&t| self.terrains.get(t))
            .map(|t| t.name.as_str())
    }

    /// Name of the object at cell `(x, y)`, or `None` if there is no object or
    /// it is out of bounds.
    pub fn object_name_at(&self, x: usize, y: usize) -> Option<&str> {
        self.content
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|c| match c {
                CellContent::Object(idx) => self.objects.get(*idx).map(|o| o.name.as_str()),
                _ => None,
            })
    }

    /// Id of the creature at cell `(x, y)`, or `None` if there is no creature
    /// or it is out of bounds.
    pub fn creature_at(&self, x: usize, y: usize) -> Option<CreatureId> {
        self.content
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|c| match c {
                CellContent::Creature(id) => Some(*id),
                _ => None,
            })
    }

    /// Current reserve of the terrain at cell `(x, y)`, read from the world's
    /// terrain catalog (not from the cell), or `None` if out of bounds. Two
    /// cells with the same terrain share the same reserve.
    pub fn reserve_at(&self, x: usize, y: usize) -> Option<i64> {
        self.terrain
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|&t| self.terrains.get(t))
            .map(|t| t.reserve)
    }

    /// Number of living creatures of the given faction, excluding `exclude`.
    /// All creatures in the world are alive (dead ones are removed on death).
    pub fn count_creatures_by_faction_excluding(
        &self,
        faction: &str,
        exclude: CreatureId,
    ) -> usize {
        self.creatures
            .iter()
            .flatten()
            .filter(|c| c.id != exclude && c.species.faction == faction)
            .count()
    }

    /// Calculates the `Nearby` structure for the given creature with a single
    /// deterministic traversal of the world: creatures by ascending id, grid by
    /// ascending rows and columns, previous-tick roars in vector order.
    /// Chebyshev distance; radius <= vision; own cell excluded from counts and
    /// from "nearest".
    pub fn nearby(&self, creature_id: CreatureId) -> Nearby {
        let self_pos = match self.creature_position(creature_id) {
            Ok(p) => p,
            Err(_) => return Nearby::empty(),
        };
        let self_creature = match self.creature(creature_id) {
            Some(c) => c,
            None => return Nearby::empty(),
        };
        let self_faction = self_creature.species.faction.as_str();
        let vision = self_creature.species.vision;

        let mut allies = 0i64;
        let mut enemies = 0i64;
        let mut objects = 0i64;
        let mut nearest_ally: Option<Nearest> = None;
        let mut nearest_enemy: Option<Nearest> = None;
        let mut nearest_object: Option<Nearest> = None;

        // Pass 1: living creatures by ascending id (excluding self).
        for c in self.creatures.iter().flatten() {
            if c.id == creature_id {
                continue;
            }
            let dx = c.x as i64 - self_pos.x as i64;
            let dy = c.y as i64 - self_pos.y as i64;
            let dist = dx.abs().max(dy.abs());
            if dist > vision {
                continue;
            }
            if c.species.faction == self_faction {
                allies += 1;
                if nearest_ally.is_none() || dist < nearest_ally.unwrap().dist {
                    nearest_ally = Some(Nearest {
                        dist,
                        dx: dx.signum(),
                        dy: dy.signum(),
                    });
                }
            } else {
                enemies += 1;
                if nearest_enemy.is_none() || dist < nearest_enemy.unwrap().dist {
                    nearest_enemy = Some(Nearest {
                        dist,
                        dx: dx.signum(),
                        dy: dy.signum(),
                    });
                }
            }
        }

        // Pass 2: grid by ascending rows, ascending columns, for objects.
        for y in 0..self.rows {
            for x in 0..self.columns {
                if let CellContent::Object(_) = &self.content[y][x] {
                    let dx = x as i64 - self_pos.x as i64;
                    let dy = y as i64 - self_pos.y as i64;
                    let dist = dx.abs().max(dy.abs());
                    if dist > vision {
                        continue;
                    }
                    objects += 1;
                    if nearest_object.is_none() || dist < nearest_object.unwrap().dist {
                        nearest_object = Some(Nearest {
                            dist,
                            dx: dx.signum(),
                            dy: dy.signum(),
                        });
                    }
                }
            }
        }

        // Pass 3: previous-tick roars in vector order.
        let mut roars = 0i64;
        let mut last_roar = String::new();
        for roar in self.roars_previous.iter() {
            if roar.creature_id == creature_id {
                continue;
            }
            let dx = roar.pos.x as i64 - self_pos.x as i64;
            let dy = roar.pos.y as i64 - self_pos.y as i64;
            let dist = dx.abs().max(dy.abs());
            if dist > vision {
                continue;
            }
            roars += 1;
            last_roar = roar.text.clone();
        }

        Nearby {
            allies,
            enemies,
            objects,
            ally: nearest_ally,
            enemy: nearest_enemy,
            object: nearest_object,
            roars,
            last_roar,
        }
    }

    /// Returns true if the cell at (x, y) is inside the map and has no object
    /// and no creature (terrain only).
    pub fn cell_is_free(&self, x: usize, y: usize) -> bool {
        if x >= self.columns || y >= self.rows {
            return false;
        }
        matches!(self.content[y][x], CellContent::Empty)
    }
}
