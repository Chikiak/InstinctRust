//! Folder loading for terrains (`.te`), objects (`.ob`), and maps (`.map`).

use crate::diagnostic::Diagnostic;
use crate::resources::map::{load_map, Map};
use crate::resources::object::{load_object, Object};
use crate::resources::terrain::{load_terrain, Terrain};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn list_sorted_files(dir: &Path, extension: &str) -> Vec<PathBuf> {
    let mut output: Vec<PathBuf> = Vec::new();
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == extension) {
                output.push(path);
            }
        }
    }
    output.sort();
    output
}

/// Loads all terrains from a folder, in alphabetical order. Duplicates of
/// `character` or `name` among terrains are reported and discarded.
pub fn load_terrains_folder(path: &Path) -> (Vec<Terrain>, Vec<Diagnostic>) {
    let mut terrains: Vec<Terrain> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let mut characters_used: Vec<char> = Vec::new();
    let mut names_used: Vec<String> = Vec::new();

    for path in list_sorted_files(path, "te") {
        let file = path.to_string_lossy().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                diagnostics.push(Diagnostic::new(
                    &file,
                    1,
                    format!("could not read the file: {e}"),
                ));
                continue;
            }
        };
        match load_terrain(&content, &file) {
            (Some(t), mut d) => {
                // Verify duplicate character and name against already loaded terrains.
                if characters_used.contains(&t.character)
                    || terrains.iter().any(|terrain| terrain.character == t.character)
                {
                    d.push(Diagnostic::new(
                        &file,
                        1,
                        format!(
                            "the character '{}' is already in use by another resource",
                            t.character
                        ),
                    ));
                    diagnostics.append(&mut d);
                    continue;
                }
                if names_used.contains(&t.name)
                    || terrains.iter().any(|terrain| terrain.name == t.name)
                {
                    d.push(Diagnostic::new(
                        &file,
                        1,
                        format!("the name '{}' is already in use by another resource", t.name),
                    ));
                    diagnostics.append(&mut d);
                    continue;
                }
                diagnostics.append(&mut d);
                characters_used.push(t.character);
                names_used.push(t.name.clone());
                terrains.push(t);
            }
            (None, mut d) => {
                diagnostics.append(&mut d);
            }
        }
    }
    (terrains, diagnostics)
}

/// Loads all objects from a folder, in alphabetical order, verifying that
/// they do not repeat `character` or `name` with terrains or objects already loaded.
pub fn load_objects_folder(path: &Path, terrains: &[Terrain]) -> (Vec<Object>, Vec<Diagnostic>) {
    let mut objects: Vec<Object> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let mut characters_used: Vec<char> = terrains.iter().map(|t| t.character).collect();
    let mut names_used: Vec<String> = terrains.iter().map(|t| clone_name(&t.name)).collect();

    for path in list_sorted_files(path, "ob") {
        let file = path.to_string_lossy().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                diagnostics.push(Diagnostic::new(
                    &file,
                    1,
                    format!("could not read the file: {e}"),
                ));
                continue;
            }
        };
        match load_object(&content, &file) {
            (Some(o), mut d) => {
                // Verify duplicate character and name against already loaded.
                if characters_used.contains(&o.character)
                    || objects.iter().any(|obj| obj.character == o.character)
                {
                    d.push(Diagnostic::new(
                        &file,
                        1,
                        format!(
                            "the character '{}' is already in use by another resource",
                            o.character
                        ),
                    ));
                    diagnostics.append(&mut d);
                    continue;
                }
                if names_used.contains(&o.name) || objects.iter().any(|obj| obj.name == o.name) {
                    d.push(Diagnostic::new(
                        &file,
                        1,
                        format!("the name '{}' is already in use by another resource", o.name),
                    ));
                    diagnostics.append(&mut d);
                    continue;
                }
                diagnostics.append(&mut d);
                characters_used.push(o.character);
                names_used.push(o.name.clone());
                objects.push(o);
            }
            (None, mut d) => {
                diagnostics.append(&mut d);
            }
        }
    }
    (objects, diagnostics)
}

fn clone_name(s: &str) -> String {
    s.to_string()
}

/// Loads all maps from a folder, in alphabetical order. Unknown characters
/// (that no terrain or object declares) produce a diagnostic with the line
/// of the row and discard that row, but not the whole map.
pub fn load_maps_folder(
    path: &Path,
    terrains: &[Terrain],
    objects: &[Object],
) -> (Vec<Map>, Vec<Diagnostic>) {
    let mut maps: Vec<Map> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let mut valid_characters: Vec<char> = terrains.iter().map(|t| t.character).collect();
    valid_characters.extend(objects.iter().map(|o| o.character));

    for path in list_sorted_files(path, "map") {
        let file = path.to_string_lossy().to_string();
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                diagnostics.push(Diagnostic::new(
                    &file,
                    1,
                    format!("could not read the file: {e}"),
                ));
                continue;
            }
        };
        match load_map(&content, &file, &valid_characters) {
            (Some(m), mut d) => {
                diagnostics.append(&mut d);
                maps.push(m);
            }
            (None, mut d) => {
                diagnostics.append(&mut d);
            }
        }
    }
    (maps, diagnostics)
}
