//! Resource loading: terrains (`.te`), objects (`.ob`), and maps (`.map`).
//!
//! Each folder is read in alphabetical file name order. A resource that
//! fails produces diagnostics and is discarded, but loading continues with
//! the others. A missing folder is treated as empty, without diagnostic.

use crate::actions::Catalog;
use crate::compile;
use crate::diagnostic::Diagnostic;
use crate::perception::PerceptionCatalog;
use crate::species::Species;
use std::fs;
use std::path::Path;
use std::rc::Rc;

/// A loaded terrain. Its reserve starts full (equal to `resource_max`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terrain {
    pub name: String,
    pub character: char,
    pub resource_max: i64,
    pub regen: i64,
    pub reserve: i64,
}

/// A loaded object. Its reserve starts at `resource_max` and does not regenerate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    pub name: String,
    pub character: char,
    pub resource_max: i64,
    pub reserve: i64,
}

/// A loaded map: its name and its rows of characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Map {
    pub name: String,
    pub rows: Vec<Vec<char>>,
}

/// Result of loading a resource directory.
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

/// Pair (result, diagnostics) for an individual resource.
type LoadResult<T> = (Option<T>, Vec<Diagnostic>);

// ---------------------------------------------------------------------------
// Key-value line parsing (.te and .ob)
// ---------------------------------------------------------------------------

/// Splits a line into (key, value) using the first whitespace as separator.
/// Returns None if the line has no key.
fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    // Comment line: its first non-blank character is '#'.
    if line.starts_with('#') {
        return None;
    }
    match line.find(char::is_whitespace) {
        Some(i) => {
            let key = &line[..i];
            let value = line[i..].trim();
            if key.is_empty() {
                None
            } else {
                Some((key, value))
            }
        }
        None => Some((line, "")),
    }
}

/// Reads all key-value pairs from a content, in order of appearance,
/// ignoring blanks and comments. Returns (pairs, lines_with_key) where
/// each entry stores the line number (from 1) for diagnostics.
fn read_keys(content: &str) -> Vec<(usize, String, String)> {
    let mut output: Vec<(usize, String, String)> = Vec::new();
    for (idx, line) in content.lines().enumerate() {
        let number = idx + 1;
        if let Some((key, value)) = split_key_value(line) {
            output.push((number, key.to_string(), value.to_string()));
        }
    }
    output
}

/// Value of a key looked up by name; returns the line number of its first
/// appearance or None if it does not exist.
fn key_value<'a>(keys: &'a [(usize, String, String)], name: &'a str) -> Option<(usize, &'a str)> {
    keys.iter()
        .find(|(_, k, _)| k == name)
        .map(|(n, _, v)| (*n, v.as_str()))
}

/// Validates that `characters` has exactly one character.
fn validate_single_character(
    number: usize,
    value: &str,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<char> {
    let mut chars = value.chars();
    let first = chars.next();
    let second = chars.next();
    match (first, second) {
        (Some(c), None) => Some(c),
        _ => {
            diagnostics.push(Diagnostic::new(
                file,
                number,
                format!("the value of 'char' must be a single character, got: '{value}'"),
            ));
            None
        }
    }
}

/// Parses an i64 integer; fails with diagnostic if not numeric.
fn parse_integer(
    number: usize,
    value: &str,
    key: &str,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match value.trim().parse::<i64>() {
        Ok(v) => Some(v),
        Err(_) => {
            diagnostics.push(Diagnostic::new(
                file,
                number,
                format!("the value of '{key}' must be an integer, got: '{value}'"),
            ));
            None
        }
    }
}

/// Loads a terrain from its in-memory content.
pub fn load_terrain(content: &str, file: &str) -> LoadResult<Terrain> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let keys = read_keys(content);

    let name = match key_value(&keys, "terrain") {
        Some((n, v)) => {
            if v.trim().is_empty() {
                diagnostics.push(Diagnostic::new(
                    file,
                    n,
                    "missing value for key 'terrain'".to_string(),
                ));
                return (None, diagnostics);
            }
            v.trim().to_string()
        }
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'terrain'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let character = match key_value(&keys, "char") {
        Some((n, v)) => match validate_single_character(n, v, file, &mut diagnostics) {
            Some(c) => c,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'char'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let resource_max = match key_value(&keys, "resource_max") {
        Some((n, v)) => match parse_integer(n, v, "resource_max", file, &mut diagnostics) {
            Some(v) => v,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'resource_max'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let regen = match key_value(&keys, "regen") {
        Some((n, v)) => match parse_integer(n, v, "regen", file, &mut diagnostics) {
            Some(v) => v,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'regen'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    if resource_max <= 0 {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            format!("'resource_max' must be greater than 0, got {resource_max}"),
        ));
        return (None, diagnostics);
    }
    if regen < 0 {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            format!("'regen' must be greater than or equal to 0, got {regen}"),
        ));
        return (None, diagnostics);
    }

    (
        Some(Terrain {
            name,
            character,
            resource_max,
            regen,
            reserve: resource_max,
        }),
        diagnostics,
    )
}

/// Loads an object from its in-memory content.
pub fn load_object(content: &str, file: &str) -> LoadResult<Object> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let keys = read_keys(content);

    let name = match key_value(&keys, "object") {
        Some((n, v)) => {
            if v.trim().is_empty() {
                diagnostics.push(Diagnostic::new(
                    file,
                    n,
                    "missing value for key 'object'".to_string(),
                ));
                return (None, diagnostics);
            }
            v.trim().to_string()
        }
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'object'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let character = match key_value(&keys, "char") {
        Some((n, v)) => match validate_single_character(n, v, file, &mut diagnostics) {
            Some(c) => c,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'char'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    let resource_max = match key_value(&keys, "resource_max") {
        Some((n, v)) => match parse_integer(n, v, "resource_max", file, &mut diagnostics) {
            Some(v) => v,
            None => return (None, diagnostics),
        },
        None => {
            diagnostics.push(Diagnostic::new(
                file,
                1,
                "missing required key 'resource_max'".to_string(),
            ));
            return (None, diagnostics);
        }
    };

    if resource_max <= 0 {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            format!("'resource_max' must be greater than 0, got {resource_max}"),
        ));
        return (None, diagnostics);
    }

    (
        Some(Object {
            name,
            character,
            resource_max,
            reserve: resource_max,
        }),
        diagnostics,
    )
}

/// Loads a map from its in-memory content. There are no comments in maps.
/// `valid_characters` are the `char` declared by terrains and objects;
/// any other character produces a diagnostic with the line of its row
/// without preventing the map or the others from loading.
pub fn load_map(content: &str, file: &str, valid_characters: &[char]) -> LoadResult<Map> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            1,
            "the map must have at least one row (the name)".to_string(),
        ));
        return (None, diagnostics);
    }

    let name = lines[0].to_string();
    let rows_str = &lines[1..];

    if rows_str.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            2,
            "the map must have at least one row of cells".to_string(),
        ));
        return (None, diagnostics);
    }

    let mut rows: Vec<Vec<char>> = Vec::new();
    let mut expected_length: Option<usize> = None;

    for (idx, row) in rows_str.iter().enumerate() {
        let number = idx + 2; // line 1 is the name
        let chars: Vec<char> = row.chars().collect();
        if let Some(l) = expected_length {
            if chars.len() != l {
                diagnostics.push(Diagnostic::new(
                    file,
                    number,
                    format!(
                        "the row has {} characters, {} were expected",
                        chars.len(),
                        l
                    ),
                ));
                continue;
            }
        } else {
            expected_length = Some(chars.len());
        }
        for c in &chars {
            if !valid_characters.contains(c) {
                diagnostics.push(Diagnostic::new(
                    file,
                    number,
                    format!("unknown character in the map: '{c}'"),
                ));
            }
        }
        rows.push(chars);
    }

    if rows.is_empty() {
        diagnostics.push(Diagnostic::new(
            file,
            2,
            "the map must have at least one row of cells".to_string(),
        ));
        return (None, diagnostics);
    }

    (Some(Map { name, rows }), diagnostics)
}

// ---------------------------------------------------------------------------
// Folder loading
// ---------------------------------------------------------------------------

fn list_sorted_files(dir: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let mut output: Vec<std::path::PathBuf> = Vec::new();
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

/// Loads all terrains from a folder, in alphabetical order.
pub fn load_terrains_folder(path: &Path) -> (Vec<Terrain>, Vec<Diagnostic>) {
    let mut terrains: Vec<Terrain> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
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
                diagnostics.append(&mut d);
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
                        format!(
                            "the name '{}' is already in use by another resource",
                            o.name
                        ),
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

/// Loads all creatures from a folder, in alphabetical order. A creature file
/// that fails to compile produces diagnostics and is skipped, but loading
/// continues with the others. A missing folder is treated as empty.
pub fn load_creatures_folder(path: &Path) -> (Vec<Species>, Vec<Diagnostic>) {
    let mut creatures: Vec<Species> = Vec::new();
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    let catalog = Catalog::default();
    let perception_catalog = PerceptionCatalog::default();

    for path in list_sorted_files(path, "ins") {
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

        let result = compile::compile(&content, &catalog, &perception_catalog, &file);

        // Only register the species when the header is complete and there are
        // no diagnostics.
        let has_errors = !result.diagnostics.is_empty();
        diagnostics.extend(result.diagnostics);

        if !has_errors {
            if let Some(header) = result.header {
                let program = Rc::new(result.program);
                creatures.push(Species::with_program(
                    header.species_name,
                    header.faction,
                    header.health,
                    header.vision,
                    header.lifespan,
                    program,
                ));
            }
        }
    }

    (creatures, diagnostics)
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
