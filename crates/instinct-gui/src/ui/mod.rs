//! eframe/egui translation layer: app state -> widgets, clicks -> messages.
//! No simulation logic lives here: no World, no step_tick, no seeds.

use crate::app::{AppState, Msg};
use crate::render::{
    pixel_to_cell, CELL_SIZE_PX, GRID_COLS, GRID_HEIGHT_PX, GRID_ROWS, GRID_WIDTH_PX,
};
use egui::{CentralPanel, Color32, Sense, Stroke, Vec2};

pub struct SpikeApp {
    pub state: AppState,
}

impl eframe::App for SpikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title("instinct-gui".to_owned()));

        let selected = match self.state.selected {
            Some(cell) => format!("Clicked cell: ({}, {})", cell.x, cell.y),
            None => "Clicked cell: none".to_string(),
        };

        let (catalog, load_lines, can_prepare) = if let Some(session) = &self.state.session {
            (
                Some((
                    session
                        .map_names()
                        .iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>(),
                    session
                        .species_names()
                        .iter()
                        .map(|n| n.to_string())
                        .collect::<Vec<_>>(),
                    session.selected_map().map(|m| m.name.clone()),
                    session.selected_species().map(|s| s.name.clone()),
                )),
                session
                    .log_entries()
                    .iter()
                    .filter(|e| e.kind == crate::app::log::LogEntryKind::Load)
                    .map(|e| e.line.clone())
                    .collect::<Vec<_>>(),
                session.can_prepare(),
            )
        } else {
            (None, Vec::new(), false)
        };

        egui::SidePanel::right("catalog_panel")
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Catálogos");

                if let Some((map_names, species_names, sel_map, sel_species)) = catalog {
                    ui.label("Mapas:");
                    for (i, name) in map_names.iter().enumerate() {
                        let selected = sel_map.as_deref() == Some(name.as_str());
                        if ui.selectable_label(selected, name.as_str()).clicked() {
                            self.state.update(Msg::MapSelected(i));
                        }
                    }

                    ui.separator();
                    ui.label("Especies:");
                    for (i, name) in species_names.iter().enumerate() {
                        let selected = sel_species.as_deref() == Some(name.as_str());
                        if ui.selectable_label(selected, name.as_str()).clicked() {
                            self.state.update(Msg::SpeciesSelected(i));
                        }
                    }

                    ui.separator();
                    ui.label("Registro de carga:");
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for line in &load_lines {
                                ui.label(line);
                            }
                        });

                    if !can_prepare {
                        ui.separator();
                        ui.label("No se puede preparar: sin mapas o sin especies.");
                    }
                } else {
                    ui.label("Sin sesión cargada.");
                }
            });

        CentralPanel::default().show(ctx, |ui| {
            ui.label(&selected);

            let (response, painter) =
                ui.allocate_painter(Vec2::new(GRID_WIDTH_PX, GRID_HEIGHT_PX), Sense::click());

            let stroke = Stroke::new(1.0, Color32::from_gray(120));
            for col in 0..=GRID_COLS {
                let x = col as f32 * CELL_SIZE_PX;
                painter.line_segment(
                    [
                        response.rect.min + egui::Vec2::new(x, 0.0),
                        response.rect.min + egui::Vec2::new(x, GRID_HEIGHT_PX),
                    ],
                    stroke,
                );
            }
            for row in 0..=GRID_ROWS {
                let y = row as f32 * CELL_SIZE_PX;
                painter.line_segment(
                    [
                        response.rect.min + egui::Vec2::new(0.0, y),
                        response.rect.min + egui::Vec2::new(GRID_WIDTH_PX, y),
                    ],
                    stroke,
                );
            }

            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let local = pos - response.rect.min;
                    if let Some(cell) = pixel_to_cell(local.x, local.y) {
                        self.state.update(Msg::CellClicked(cell));
                    }
                }
            }
        });
    }
}
