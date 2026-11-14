//! ASCII drawing of the world.

use crate::diagnostic::Diagnostic;
use super::creature::CellContent;
use super::World;

impl World {
    /// Draws the world in ASCII: the creature (first letter of its species)
    /// if there is one, otherwise the object's character if there is one,
    /// otherwise the terrain's character.
    ///
    /// Validates against the world's own catalogs (sole owner), so it does not
    /// receive catalogs as arguments.
    ///
    /// Returns a `Diagnostic` when the grid references a creature that does
    /// not exist, or when an object/terrain index in the grid is not covered
    /// by the world's catalogs.
    pub fn draw(&self) -> Result<String, Diagnostic> {
        let mut output = String::new();
        for row in 0..self.rows {
            for column in 0..self.columns {
                let c = match &self.content[row][column] {
                    CellContent::Creature(id) => {
                        let creature = self.creature(*id).ok_or_else(|| {
                            Diagnostic::new(
                                "world",
                                0,
                                format!("grid references missing creature {id:?}"),
                            )
                        })?;
                        creature.species.name.chars().next().ok_or_else(|| {
                            Diagnostic::new(
                                "world",
                                0,
                                format!("creature {id:?} has an empty species name"),
                            )
                        })?
                    }
                    CellContent::Object(idx) => {
                        if *idx >= self.objects.len() {
                            return Err(Diagnostic::new(
                                "world",
                                0,
                                format!(
                                    "grid references object index {idx} but the object catalog has {} entries",
                                    self.objects.len()
                                ),
                            ));
                        }
                        self.objects[*idx].character
                    }
                    CellContent::Empty => {
                        let t_idx = self.terrain[row][column];
                        if t_idx >= self.terrains.len() {
                            return Err(Diagnostic::new(
                                "world",
                                0,
                                format!(
                                    "grid references terrain index {t_idx} but the terrain catalog has {} entries",
                                    self.terrains.len()
                                ),
                            ));
                        }
                        self.terrains[t_idx].character
                    }
                };
                output.push(c);
            }
            output.push('\n');
        }
        Ok(output)
    }
}
