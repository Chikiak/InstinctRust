//! Read-only snapshot of a creature and its world, used to evaluate perceptions.

use super::creature::{CellContent, CreatureId};
use super::World;
use crate::perceptions::{CreatureView, Nearby};

/// Read-only view of a creature and its world, used to evaluate perceptions.
/// Nothing here mutates the world except `random`, which advances the
/// generator through interior mutability.
#[derive(Debug)]
pub struct CreatureSnapshot<'a> {
    pub(in crate::world) world: &'a World,
    pub(in crate::world) creature_id: CreatureId,
}

impl CreatureView for CreatureSnapshot<'_> {
    fn health(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.health)
            .unwrap_or(0)
    }

    fn age(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.age)
            .unwrap_or(0)
    }

    fn lifespan(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.lifespan)
            .unwrap_or(0)
    }

    fn x(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.x as i64)
            .unwrap_or(0)
    }

    fn y(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.y as i64)
            .unwrap_or(0)
    }

    fn tick(&self) -> u64 {
        self.world.ticks
    }

    fn random(&self) -> i64 {
        self.world.rng.borrow_mut().next_bounded(99)
    }

    fn vision(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.species.vision)
            .unwrap_or(0)
    }

    fn species_name(&self) -> &str {
        self.world
            .creature(self.creature_id)
            .map(|c| c.species.name.as_str())
            .unwrap_or("")
    }

    fn faction_name(&self) -> &str {
        self.world
            .creature(self.creature_id)
            .map(|c| c.species.faction.as_str())
            .unwrap_or("")
    }

    fn terrain_here(&self) -> &str {
        self.world
            .creature(self.creature_id)
            .and_then(|c| self.world.creature_position(c.id).ok())
            .and_then(|p| self.world.terrain_name_at(p.x, p.y))
            .unwrap_or("")
    }

    fn reserve_here(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .and_then(|c| self.world.creature_position(c.id).ok())
            .and_then(|p| self.world.reserve_at(p.x, p.y))
            .unwrap_or(0)
    }

    fn width(&self) -> i64 {
        self.world.columns as i64
    }

    fn height(&self) -> i64 {
        self.world.rows as i64
    }

    fn total_creatures(&self) -> i64 {
        // Only counts living creatures: free slots (`None`) are not creatures.
        let n = self.world.creatures.iter().flatten().count();
        let present = self.world.creature(self.creature_id).is_some();
        (n.saturating_sub(if present { 1 } else { 0 })) as i64
    }

    fn total_allies(&self) -> i64 {
        let Some(creature) = self.world.creature(self.creature_id) else {
            return 0;
        };
        self.world
            .count_creatures_by_faction_excluding(&creature.species.faction, self.creature_id)
            as i64
    }

    fn total_enemies(&self) -> i64 {
        // Everything that is not an ally (and not the creature itself) is an enemy.
        self.total_creatures() - self.total_allies()
    }

    fn nearby(&self) -> Nearby {
        self.world.nearby(self.creature_id)
    }

    fn see(&self, x: i64, y: i64) -> i64 {
        // Radius check first: Chebyshev distance from creature's cell.
        let self_pos = match self.world.creature_position(self.creature_id) {
            Ok(p) => p,
            Err(_) => return -1,
        };
        let self_creature = match self.world.creature(self.creature_id) {
            Some(c) => c,
            None => return -1,
        };
        let dx = x - self_pos.x as i64;
        let dy = y - self_pos.y as i64;
        let dist = dx.abs().max(dy.abs());
        if dist > self_creature.species.vision {
            return -1;
        }

        if x < 0 || y < 0 || x >= self.world.columns as i64 || y >= self.world.rows as i64 {
            return 1;
        }
        let x = x as usize;
        let y = y as usize;

        match &self.world.content[y][x] {
            CellContent::Creature(target_id) => {
                let target = match self.world.creature(*target_id) {
                    Some(c) => c,
                    None => return 0,
                };
                if target.species.faction == self_creature.species.faction {
                    2
                } else {
                    3
                }
            }
            CellContent::Object(_) => 1,
            CellContent::Empty => 0,
        }
    }

    fn name(&self, x: i64, y: i64) -> &str {
        // Out of radius or out of map: empty text.
        let self_pos = match self.world.creature_position(self.creature_id) {
            Ok(p) => p,
            Err(_) => return "",
        };
        let self_creature = match self.world.creature(self.creature_id) {
            Some(c) => c,
            None => return "",
        };
        let dx = x - self_pos.x as i64;
        let dy = y - self_pos.y as i64;
        let dist = dx.abs().max(dy.abs());
        if dist > self_creature.species.vision {
            return "";
        }

        if x < 0 || y < 0 || x >= self.world.columns as i64 || y >= self.world.rows as i64 {
            return "";
        }
        let x = x as usize;
        let y = y as usize;

        match &self.world.content[y][x] {
            CellContent::Creature(target_id) => self
                .world
                .creature(*target_id)
                .map(|c| c.species.name.as_str())
                .unwrap_or(""),
            CellContent::Object(obj_idx) => self.world.objects[*obj_idx].name.as_str(),
            CellContent::Empty => self.world.terrain_name_at(x, y).unwrap_or(""),
        }
    }
}
