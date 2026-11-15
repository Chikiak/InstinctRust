use crate::USAGE;

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::perceptions::PerceptionCatalog;

/// Executes the `check` subcommand. Returns the exit code.
pub(crate) fn execute_check(path: &str) -> i32 {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{USAGE}");
            eprintln!("error: could not read file '{path}': {e}");
            return 1;
        }
    };

    let catalog = Catalog::default();
    let perception_catalog = PerceptionCatalog::default();
    let result = compile::compile(&content, &catalog, &perception_catalog, path);

    if result.diagnostics.is_empty() {
        println!("OK");
        0
    } else {
        for d in &result.diagnostics {
            eprintln!("{d}");
        }
        1
    }
}
