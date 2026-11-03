//! World model: terrain grid, objects, and ASCII drawing.

use crate::diagnostic::Diagnostic;
use crate::resources::{Map, Object, Terrain};

/// An element occupying a cell on the terrain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellContent {
    /// Terrain only.
    Empty,
    /// There is an object on the terrain.
    Object(usize), // index in the world's object catalog
}

/// The world: terrain grid and objects on top.
#[derive(Debug, Clone)]
pub struct World {
    pub map_name: String,
    pub columns: usize,
    pub rows: usize,
    /// Terrain of each cell, indexed as [row][column].
    pub terrain: Vec<Vec<usize>>, // index in the terrain catalog
    /// Optional content (object) of each cell.
    pub content: Vec<Vec<CellContent>>,
    pub diagnostics: Vec<Diagnostic>,
}

impl World {
    /// Builds a world from a loaded map and the catalogs.
    /// Characters that do not match any terrain or object produce a diagnostic;
    /// the terrain below a cell with an object is the first loaded terrain.
    pub fn from_map(map: &Map, terrains: &[Terrain], objects: &[Object]) -> Self {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        if terrains.is_empty() {
            diagnostics.push(Diagnostic::new(
                map.name.as_str(),
                1,
                "no terrains loaded to draw the map".to_string(),
            ));
            return World {
                map_name: map.name.clone(),
                columns: 0,
                rows: 0,
                terrain: Vec::new(),
                content: Vec::new(),
                diagnostics,
            };
        }

        let first_terrain = 0usize;
        let columns = map.rows[0].len();
        let rows = map.rows.len();

        let mut terrain: Vec<Vec<usize>> = Vec::with_capacity(rows);
        let mut content: Vec<Vec<CellContent>> = Vec::with_capacity(rows);

        for (row_idx, row) in map.rows.iter().enumerate() {
            let mut row_terrain: Vec<usize> = Vec::with_capacity(columns);
            let mut row_content: Vec<CellContent> = Vec::with_capacity(columns);
            for c in row.iter() {
                // Look for the character among objects first, then terrains.
                let object_idx = objects.iter().position(|o| o.character == *c);
                if let Some(idx) = object_idx {
                    row_terrain.push(first_terrain);
                    row_content.push(CellContent::Object(idx));
                    continue;
                }
                let terrain_idx = terrains.iter().position(|t| t.character == *c);
                match terrain_idx {
                    Some(idx) => {
                        row_terrain.push(idx);
                        row_content.push(CellContent::Empty);
                    }
                    None => {
                        diagnostics.push(Diagnostic::new(
                            map.name.as_str(),
                            row_idx + 2,
                            format!("unknown character in the map: '{c}'"),
                        ));
                        row_terrain.push(first_terrain);
                        row_content.push(CellContent::Empty);
                    }
                }
            }
            terrain.push(row_terrain);
            content.push(row_content);
        }

        World {
            map_name: map.name.clone(),
            columns,
            rows,
            terrain,
            content,
            diagnostics,
        }
    }

    /// Draws the world in ASCII: the character of the object if there is one, otherwise the
    /// terrain's character. Creatures will be added in F0-04.
    pub fn draw(&self, terrains: &[Terrain], objects: &[Object]) -> String {
        let mut output = String::new();
        for row in 0..self.rows {
            for column in 0..self.columns {
                let c = match &self.content[row][column] {
                    CellContent::Object(idx) => objects[*idx].character,
                    CellContent::Empty => terrains[self.terrain[row][column]].character,
                };
                output.push(c);
            }
            output.push('\n');
        }
        output
    }
}
