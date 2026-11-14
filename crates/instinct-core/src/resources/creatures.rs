//! Loading of creature species (`.ins`).

use crate::actions::Catalog;
use crate::compile;
use crate::diagnostic::Diagnostic;
use crate::perceptions::PerceptionCatalog;
use crate::resources::folder::list_sorted_files;
use crate::species::Species;
use std::fs;
use std::path::Path;
use std::rc::Rc;

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
