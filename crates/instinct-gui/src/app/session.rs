//! Headless session: loaded resources catalog (maps + species), a selection
//! over them, and a log of load diagnostics. No eframe/egui knowledge.

use crate::app::log::{LogBuffer, LogEntry};
use instinct_core::diagnostic::Diagnostic;
use instinct_core::resources::{load_resources, LoadedResources, Map};
use instinct_core::species::Species;
use std::path::Path;

/// A loaded resources session: catalogs, current selection, and load log.
pub struct LoadedSession {
    resources: LoadedResources,
    log: LogBuffer,
    selected_map_idx: usize,
    selected_species_idx: usize,
}

impl LoadedSession {
    /// Loads every resource under `root`, pushes each load diagnostic into the
    /// log, and selects the first map and the first species (index 0) when the
    /// corresponding catalog is non-empty.
    pub fn new(root: &Path) -> Self {
        let mut resources = load_resources(root);
        // Orden único: el índice que ve la UI es el índice del catálogo.
        resources.maps.sort_by(|a, b| a.name.cmp(&b.name));
        resources.creatures.sort_by(|a, b| a.name.cmp(&b.name));

        let mut log = LogBuffer::default();
        for diagnostic in &resources.diagnostics {
            log.push_diagnostic(diagnostic);
        }

        let selected_map_idx = 0;
        let selected_species_idx = 0;

        if resources.maps.is_empty() {
            log.push_diagnostic(&Diagnostic::new(
                root.display().to_string(),
                1,
                "no maps".to_string(),
            ));
        }
        if resources.creatures.is_empty() {
            log.push_diagnostic(&Diagnostic::new(
                root.display().to_string(),
                1,
                "no species".to_string(),
            ));
        }

        LoadedSession {
            resources,
            log,
            selected_map_idx,
            selected_species_idx,
        }
    }

    /// All loaded resources (terrains, objects, maps, creatures).
    pub fn resources(&self) -> &LoadedResources {
        &self.resources
    }

    /// All loaded maps, in load order.
    pub fn maps(&self) -> &[Map] {
        &self.resources.maps
    }

    /// All loaded species, in load order.
    pub fn species(&self) -> &[Species] {
        &self.resources.creatures
    }

    /// Map names sorted alphabetically.
    pub fn map_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .resources
            .maps
            .iter()
            .map(|m| m.name.as_str())
            .collect();
        names.sort();
        names
    }

    /// Species names sorted alphabetically.
    pub fn species_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self
            .resources
            .creatures
            .iter()
            .map(|s| s.name.as_str())
            .collect();
        names.sort();
        names
    }

    /// The currently selected map index (into `maps()`).
    pub fn selected_map_index(&self) -> usize {
        self.selected_map_idx
    }

    /// The currently selected map, if any.
    pub fn selected_map(&self) -> Option<&Map> {
        self.resources.maps.get(self.selected_map_idx)
    }

    /// The currently selected species, if any.
    pub fn selected_species(&self) -> Option<&Species> {
        self.resources.creatures.get(self.selected_species_idx)
    }

    /// Selects the map at `idx`, clamped to the catalog length. No-op when empty.
    pub fn select_map(&mut self, idx: usize) {
        if self.resources.maps.is_empty() {
            return;
        }
        self.selected_map_idx = idx.min(self.resources.maps.len() - 1);
    }

    /// Selects the species at `idx`, clamped to the catalog length. No-op when empty.
    pub fn select_species(&mut self, idx: usize) {
        if self.resources.creatures.is_empty() {
            return;
        }
        self.selected_species_idx = idx.min(self.resources.creatures.len() - 1);
    }

    /// True when both catalogs are non-empty: the app may enter preparation.
    pub fn can_prepare(&self) -> bool {
        !self.resources.maps.is_empty() && !self.resources.creatures.is_empty()
    }

    /// All log entries, oldest first (delegates to the inner `LogBuffer`).
    pub fn log_entries(&self) -> &[LogEntry] {
        self.log.entries()
    }
}
