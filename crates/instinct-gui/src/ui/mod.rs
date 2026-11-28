//! Capa egui: dibuja el estado y traduce interacciones en `Msg`.
//! Los paneles no modifican el estado: acumulan mensajes y `update` los
//! aplica al final del frame. Sin lógica de simulación.

use crate::app::log::LogEntryKind;
use crate::app::time::{RunState, Speed};
use crate::app::{AppState, Msg};
use crate::render::{
    cell_to_pixel, grid_from_snapshot, roar_markers, Cell, Rgb, CELL_SIZE_PX, MARGIN_PX,
};
use egui::{
    Align2, CentralPanel, Color32, FontId, Painter, Pos2, Rect, Sense, SidePanel, Stroke,
    StrokeKind, Vec2,
};
use instinct_core::event::Event;
use instinct_core::world::{CreatureId, Roar, WorldSnapshot};

/// Eventos mostrados en el panel (el buffer completo sigue en memoria).
const EVENTS_SHOWN: usize = 300;
/// Tope del delta por frame: evita ráfagas de ticks tras una pausa larga.
const MAX_FRAME_DT_MS: f64 = 250.0;

#[derive(Default)]
pub struct SpikeApp {
    pub state: AppState,
    /// Tiempo absoluto del frame anterior (segundos), para calcular el delta.
    pub last_time: Option<f64>,
}

impl eframe::App for SpikeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Title("instinct-gui".to_owned()));

        let mut msgs: Vec<Msg> = Vec::new();

        // Reloj: el delta alimenta el acumulador de TimeControl.
        let now = ctx.input(|i| i.time);
        if let Some(last) = self.last_time {
            let dt_ms = ((now - last) * 1000.0).clamp(0.0, MAX_FRAME_DT_MS) as u64;
            if dt_ms > 0 {
                msgs.push(Msg::TickAdvance(dt_ms));
            }
        }
        self.last_time = Some(now);

        let state = &self.state;

        SidePanel::left("left_panel")
            .default_width(290.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    preparation_section(ui, state, &mut msgs);
                    ui.separator();
                    time_section(ui, state, &mut msgs);
                });
            });

        SidePanel::right("right_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                catalog_section(ui, state, &mut msgs);
                ui.separator();
                events_section(ui, state);
            });

        CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                grid_section(ui, state, &mut msgs);
            });
        });

        for msg in msgs {
            self.state.update(msg);
        }

        // egui solo repinta ante eventos: mientras corre, pedimos el siguiente frame.
        if matches!(self.state.time.state, RunState::Running(_)) {
            ctx.request_repaint();
        }
    }
}

// --- Paneles -----------------------------------------------------------------

fn preparation_section(ui: &mut egui::Ui, state: &AppState, msgs: &mut Vec<Msg>) {
    ui.heading("Preparación");

    let Some(session) = state.session.as_ref() else {
        ui.label("Sin sesión cargada.");
        return;
    };

    let Some(prep) = state.prep_state.as_ref() else {
        if session.can_prepare() {
            if ui.button("Entrar en preparación").clicked() {
                msgs.push(Msg::PrepEnter(session.selected_map_index()));
            }
        } else {
            ui.label("Carga mapas y especies para preparar.");
        }
        return;
    };

    let locked = matches!(state.time.state, RunState::Running(_) | RunState::Paused(_));

    ui.label(format!("Mapa: {}", prep.recipe.map_name));
    ui.label(format!(
        "Semilla: {}  ·  Colocadas: {}",
        prep.recipe.seed,
        prep.placed_count()
    ));
    if locked {
        ui.colored_label(
            Color32::from_rgb(230, 170, 60),
            "Simulación en curso: reinicia para editar la colocación.",
        );
    }

    ui.add_space(4.0);
    ui.label("Especie a colocar:");
    let species = session.species_names();
    let current = species
        .get(state.prep_selected_species)
        .copied()
        .unwrap_or("—");
    egui::ComboBox::new("prep_species", "")
        .selected_text(current)
        .show_ui(ui, |ui| {
            for (i, name) in species.iter().enumerate() {
                if ui
                    .selectable_label(state.prep_selected_species == i, *name)
                    .clicked()
                {
                    msgs.push(Msg::PrepSelectSpecies(i));
                }
            }
        });

    ui.separator();
    ui.label("Colocadas:");
    for (pid, name, x, y) in prep.creatures_in_prep() {
        ui.horizontal(|ui| {
            ui.label(format!("{name} @ {x},{y}"));
            if !locked && ui.small_button("Quitar").clicked() {
                msgs.push(Msg::PrepRemove(pid));
            }
        });
    }
}

fn time_section(ui: &mut egui::Ui, state: &AppState, msgs: &mut Vec<Msg>) {
    ui.heading("Tiempo");

    let status = match state.time.state {
        RunState::Idle => "Idle".to_string(),
        RunState::Running(s) => format!("Corriendo a {} tps", s.tps()),
        RunState::Paused(s) => format!("Pausado ({} tps)", s.tps()),
    };
    ui.label(format!("{status}  ·  tick {}", state.time.tick_count()));

    ui.horizontal_wrapped(|ui| {
        let play_label = match state.time.state {
            RunState::Idle => "Play",
            RunState::Running(_) => "Corriendo",
            RunState::Paused(_) => "Reanudar",
        };
        if ui.button(play_label).clicked() {
            msgs.push(Msg::TickPlay);
        }
        if ui.button("Pausa").clicked() {
            msgs.push(Msg::TickPause);
        }
        if ui.button("Paso").clicked() {
            msgs.push(Msg::TickStep);
        }
        if ui.button("Reiniciar").clicked() {
            msgs.push(Msg::Restart);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Velocidad:");
        for speed in [Speed::T2, Speed::T10, Speed::T30] {
            if ui
                .selectable_label(state.time.speed == speed, format!("{} tps", speed.tps()))
                .clicked()
            {
                msgs.push(Msg::TickSetSpeed(speed));
            }
        }
    });
}

fn catalog_section(ui: &mut egui::Ui, state: &AppState, msgs: &mut Vec<Msg>) {
    let Some(session) = state.session.as_ref() else {
        return;
    };

    ui.heading("Catálogos");

    ui.label("Mapas:");
    let current_map = session.selected_map_index();
    for (i, name) in session.map_names().iter().enumerate() {
        if ui.selectable_label(i == current_map, *name).clicked() {
            msgs.push(Msg::MapSelected(i));
        }
    }

    ui.separator();
    ui.label("Especies:");
    for (i, name) in session.species_names().iter().enumerate() {
        if ui
            .selectable_label(state.prep_selected_species == i, *name)
            .clicked()
        {
            msgs.push(Msg::PrepSelectSpecies(i));
        }
    }

    let load_lines: Vec<&str> = session
        .log_entries()
        .iter()
        .filter(|e| e.kind == LogEntryKind::Load)
        .map(|e| e.line.as_str())
        .collect();
    if !load_lines.is_empty() {
        ui.collapsing("Registro de carga", |ui| {
            for line in load_lines {
                ui.label(line);
            }
        });
    }
}

fn events_section(ui: &mut egui::Ui, state: &AppState) {
    ui.heading("Eventos");

    let Some(buffer) = state.log_buffer.as_ref() else {
        ui.label("Aún no hay simulación.");
        return;
    };
    let buffer = buffer.borrow();
    let events: Vec<&str> = buffer
        .entries()
        .iter()
        .filter(|e| e.kind == LogEntryKind::Event)
        .map(|e| e.line.as_str())
        .collect();
    let start = events.len().saturating_sub(EVENTS_SHOWN);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            if events.is_empty() {
                ui.label("Sin eventos todavía.");
            }
            for line in &events[start..] {
                ui.monospace(*line);
            }
        });
}

fn grid_section(ui: &mut egui::Ui, state: &AppState, msgs: &mut Vec<Msg>) {
    let snapshot = state
        .sim
        .as_ref()
        .map(|sim| sim.world().snapshot())
        .or_else(|| state.prep_state.as_ref().map(|prep| prep.world.snapshot()));
    let Some(snapshot) = snapshot else {
        ui.label("Carga un mapa y entra en preparación.");
        return;
    };

    ui.label(format!("{}  ·  tick {}", snapshot.map_name, snapshot.ticks));
    ui.add_space(6.0);

    // Tamaño según el mundo real, no según constantes fijas.
    let cols = snapshot.columns;
    let rows = snapshot.rows;
    let size = Vec2::new(
        cols as f32 * CELL_SIZE_PX + 2.0 * MARGIN_PX,
        rows as f32 * CELL_SIZE_PX + 2.0 * MARGIN_PX,
    );
    let (response, painter) = ui.allocate_painter(size, Sense::click());
    let origin = response.rect.min + Vec2::splat(MARGIN_PX);

    draw_cells(&painter, origin, &snapshot);
    draw_roars(&painter, origin, state, &snapshot);
    draw_lines(&painter, origin, cols, rows);

    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let local = pos - origin;
            if let Some(cell) = hit_cell(local, cols, rows) {
                handle_grid_click(state, cell, msgs);
            }
        }
    }
}

/// Celda bajo un punto relativo al origen de la rejilla (sin margen).
fn hit_cell(local: Vec2, cols: usize, rows: usize) -> Option<Cell> {
    let x = (local.x / CELL_SIZE_PX).floor();
    let y = (local.y / CELL_SIZE_PX).floor();
    if x < 0.0 || y < 0.0 || x as usize >= cols || y as usize >= rows {
        return None;
    }
    Some(Cell {
        x: x as usize,
        y: y as usize,
    })
}

// --- Interacción -------------------------------------------------------------

/// Solo la preparación modifica el mundo: quita si hay criatura, coloca si no.
fn handle_grid_click(state: &AppState, cell: Cell, msgs: &mut Vec<Msg>) {
    let Some(prep) = state.prep_state.as_ref() else {
        return;
    };
    if matches!(state.time.state, RunState::Running(_) | RunState::Paused(_)) {
        return;
    }
    if prep.world.creature_at(cell.x, cell.y).is_some() {
        if let Some(pid) = prep.placement_id_at(cell.x, cell.y) {
            msgs.push(Msg::PrepRemove(pid));
        }
    } else {
        msgs.push(Msg::PrepPlace(state.prep_selected_species, cell.x, cell.y));
    }
}

// --- Dibujo de la rejilla ----------------------------------------------------

fn cell_rect(origin: Pos2, cell: Cell) -> Rect {
    let (px, py) = cell_to_pixel(cell);
    Rect::from_min_size(origin + Vec2::new(px, py), Vec2::splat(CELL_SIZE_PX))
}

fn contrast_text(c: Rgb) -> Color32 {
    let luminance = (c.r as f32 * 0.299 + c.g as f32 * 0.587 + c.b as f32 * 0.114) / 255.0;
    if luminance > 0.5 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

fn draw_cells(painter: &Painter, origin: Pos2, snapshot: &WorldSnapshot) {
    for row in grid_from_snapshot(snapshot).cells {
        for view in row {
            let rect = cell_rect(
                origin,
                Cell {
                    x: view.x,
                    y: view.y,
                },
            );
            let Rgb { r, g, b } = view.color;
            painter.rect_filled(rect, 0.0, Color32::from_rgb(r, g, b));
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                view.symbol.to_string(),
                FontId::monospace(CELL_SIZE_PX * 0.8),
                contrast_text(view.color),
            );
        }
    }
}

fn draw_roars(painter: &Painter, origin: Pos2, state: &AppState, snapshot: &WorldSnapshot) {
    // Rugidos del último tick (desde los eventos) y del tick anterior (desde el mundo).
    let mut roars: Vec<Roar> = Vec::new();
    for ev in &state.last_events {
        if let Event::Roared {
            tick,
            species,
            pos,
            text,
        } = ev
        {
            if *tick == snapshot.ticks.saturating_sub(1) {
                roars.push(Roar {
                    tick: *tick,
                    creature_id: CreatureId(0), // el marcador solo usa posición y texto
                    species: species.clone(),
                    pos: *pos,
                    text: text.clone(),
                });
            }
        }
    }
    if let Some(sim) = state.sim.as_ref() {
        roars.extend(sim.world().previous_roars().iter().cloned());
    }

    for (x, y, text) in roar_markers(&roars, snapshot.rows, snapshot.columns) {
        let rect = cell_rect(origin, Cell { x, y });
        painter.rect_filled(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(210, 160, 40, 120),
        );
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(2.0_f32, Color32::from_rgb(210, 120, 0)),
            StrokeKind::Inside,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            "!",
            FontId::monospace(CELL_SIZE_PX * 0.9),
            Color32::WHITE,
        );
        painter.text(
            rect.center() + Vec2::new(0.0, CELL_SIZE_PX * 0.35),
            Align2::CENTER_TOP,
            text,
            FontId::monospace(CELL_SIZE_PX * 0.45),
            Color32::from_rgb(255, 200, 90),
        );
    }
}

fn draw_lines(painter: &Painter, origin: Pos2, cols: usize, rows: usize) {
    let width = cols as f32 * CELL_SIZE_PX;
    let height = rows as f32 * CELL_SIZE_PX;
    let stroke = Stroke::new(1.0_f32, Color32::from_gray(120));
    for col in 0..=cols {
        let x = col as f32 * CELL_SIZE_PX;
        painter.line_segment(
            [origin + Vec2::new(x, 0.0), origin + Vec2::new(x, height)],
            stroke,
        );
    }
    for row in 0..=rows {
        let y = row as f32 * CELL_SIZE_PX;
        painter.line_segment(
            [origin + Vec2::new(0.0, y), origin + Vec2::new(width, y)],
            stroke,
        );
    }
}
