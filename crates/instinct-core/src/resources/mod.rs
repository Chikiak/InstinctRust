//! Resource loading: terrains (`.te`), objects (`.ob`), and maps (`.map`).
//!
//! Each folder is read in alphabetical file name order. A resource that
//! fails produces diagnostics and is discarded, but loading continues with
//! the others. A missing folder is treated as empty, without diagnostic.

mod creatures;
mod folder;
mod keys;
mod map;
mod object;
mod terrain;

pub use self::creatures::load_creatures_folder;
pub use self::folder::{load_maps_folder, load_objects_folder, load_terrains_folder};
pub use self::map::{load_map, Map};
pub use self::object::{load_object, Object};
pub use self::terrain::{load_terrain, Terrain};

use crate::diagnostic::Diagnostic;
use std::path::Path;

/// The result of loading a resource directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedResources {
    pub terrains: Vec<Terrain>,
    pub objects: Vec<Object>,
    pub maps: Vec<Map>,
    pub creatures: Vec<crate::species::Species>,
    pub diagnostics: Vec<Diagnostic>,
}

impl LoadedResources {
    pub fn empty() -> Self {
        LoadedResources {
            terrains: Vec::new(),
            objects: Vec::new(),
            maps: Vec::new(),
            creatures: Vec::new(),
            diagnostics: Vec::new(),
        }
    }
}

/// Loads all resources from a root directory: `terrains/`, `objects/`,
/// `maps/` and `creatures/` in that order. Missing folders are treated as empty.
pub fn load_resources(root: &Path) -> LoadedResources {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let dir_terrains = root.join("terrains");
    let (terrains, mut d) = load_terrains_folder(&dir_terrains);
    diagnostics.append(&mut d);

    let dir_objects = root.join("objects");
    let (objects, mut d) = load_objects_folder(&dir_objects, &terrains);
    diagnostics.append(&mut d);

    let dir_maps = root.join("maps");
    let (maps, mut d) = load_maps_folder(&dir_maps, &terrains, &objects);
    diagnostics.append(&mut d);

    let dir_creatures = root.join("creatures");
    let (creatures, mut d) = load_creatures_folder(&dir_creatures);
    diagnostics.append(&mut d);

    LoadedResources {
        terrains,
        objects,
        maps,
        creatures,
        diagnostics,
    }
}
