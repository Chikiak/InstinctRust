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

/// Opciones de carga para `load_resources_folder`.
struct FolderLoadOptions {
    extension: &'static str,
    existing_characters: Vec<char>,
    existing_names: Vec<String>,
    check_duplicates: bool,
}

/// Generic loader for resources in a folder, in alphabetical path order.
///
/// `existing_characters` / `existing_names` are values already taken by
/// resources loaded before (empty for terrains, the terrains' ones for objects).
/// When `check_duplicates` is true, a repeated `character` or `name` produces
/// a diagnostic and the file is discarded.
fn load_resources_folder<T, F, C, N>(
    path: &Path,
    options: FolderLoadOptions,
    load: F,
    character_of: C,
    name_of: N,
) -> (Vec<T>, Vec<Diagnostic>)
where
    F: Fn(&str, &str) -> (Option<T>, Vec<Diagnostic>),
    C: Fn(&T) -> char,
    N: Fn(&T) -> String,
{
    let FolderLoadOptions {
        extension,
        existing_characters,
        existing_names,
        check_duplicates,
    } = options;
    let mut resources: Vec<T> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let mut characters_used: Vec<char> = existing_characters;
    let mut names_used: Vec<String> = existing_names;

    for path in list_sorted_files(path, extension) {
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
        match load(&content, &file) {
            (Some(resource), mut d) => {
                if check_duplicates {
                    let character = character_of(&resource);
                    let name = name_of(&resource);
                    if characters_used.contains(&character)
                        || resources.iter().any(|r| character_of(r) == character)
                    {
                        d.push(Diagnostic::new(
                            &file,
                            1,
                            format!(
                                "the character '{}' is already in use by another resource",
                                character
                            ),
                        ));
                        diagnostics.append(&mut d);
                        continue;
                    }
                    if names_used.contains(&name) || resources.iter().any(|r| name_of(r) == name) {
                        d.push(Diagnostic::new(
                            &file,
                            1,
                            format!("the name '{}' is already in use by another resource", name),
                        ));
                        diagnostics.append(&mut d);
                        continue;
                    }
                    characters_used.push(character);
                    names_used.push(name);
                }
                diagnostics.append(&mut d);
                resources.push(resource);
            }
            (None, mut d) => {
                diagnostics.append(&mut d);
            }
        }
    }
    (resources, diagnostics)
}

/// Loads all terrains from a folder, in alphabetical order. Duplicates of
/// `character` or `name` among terrains are reported and discarded.
pub fn load_terrains_folder(path: &Path) -> (Vec<Terrain>, Vec<Diagnostic>) {
    load_resources_folder(
        path,
        FolderLoadOptions {
            extension: "te",
            existing_characters: Vec::new(),
            existing_names: Vec::new(),
            check_duplicates: true,
        },
        load_terrain,
        |terrain| terrain.character,
        |terrain| terrain.name.clone(),
    )
}

/// Loads all objects from a folder, in alphabetical order, verifying that
/// they do not repeat `character` or `name` with terrains or objects already loaded.
pub fn load_objects_folder(path: &Path, terrains: &[Terrain]) -> (Vec<Object>, Vec<Diagnostic>) {
    let existing_characters: Vec<char> = terrains.iter().map(|t| t.character).collect();
    let existing_names: Vec<String> = terrains.iter().map(|t| clone_name(&t.name)).collect();
    load_resources_folder(
        path,
        FolderLoadOptions {
            extension: "ob",
            existing_characters,
            existing_names,
            check_duplicates: true,
        },
        load_object,
        |object| object.character,
        |object| clone_name(&object.name),
    )
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
    let mut valid_characters: Vec<char> = terrains.iter().map(|t| t.character).collect();
    valid_characters.extend(objects.iter().map(|o| o.character));
    load_resources_folder(
        path,
        FolderLoadOptions {
            extension: "map",
            existing_characters: Vec::new(),
            existing_names: Vec::new(),
            check_duplicates: false,
        },
        move |content, file| load_map(content, file, &valid_characters),
        // No duplicate validation for maps: these closures are never invoked.
        |_map| '\0',
        |_map| String::new(),
    )
}
