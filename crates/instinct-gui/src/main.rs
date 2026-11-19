use instinct_gui::{app::AppState, render, ui::SpikeApp};
use std::process::exit;

/// Usage text printed to stderr (exit code 2) for unknown arguments.
const USAGE: &str = "Usage: instinct-gui [--smoke]\n\nOptions:\n  --smoke    headless flow without opening a window (implemented in F5-08).";

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 && args[1] == "--smoke" {
        eprintln!("instinct-gui: --smoke accepted (headless flow implemented in F5-08)");
        return Ok(());
    }

    // Unknown arguments: usage to stderr and exit code 2 (instinct-cli style).
    if args.len() > 1 {
        eprintln!("{USAGE}");
        exit(2);
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([
            render::GRID_WIDTH_PX + 2.0 * render::MARGIN_PX,
            render::GRID_HEIGHT_PX + 2.0 * render::MARGIN_PX,
        ]),
        ..Default::default()
    };
    eframe::run_native(
        "instinct-gui",
        options,
        Box::new(|_cc| {
            Ok(Box::new(SpikeApp {
                state: AppState::default(),
            }))
        }),
    )
}
