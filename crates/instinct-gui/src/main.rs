use instinct_core::event::WorldOutput;
use instinct_core::resources;
use instinct_core::sim::Simulation;
use instinct_core::world;
use instinct_gui::{app::session::LoadedSession, app::AppState, render, ui::SpikeApp};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::rc::Rc;

/// Root of `examples`, resolved from this crate's manifest dir so `--smoke`
/// works regardless of the current directory.
fn examples_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// Usage text printed to stderr (exit code 2) for unknown arguments.
const USAGE: &str = "Usage: instinct-gui [--smoke] [--root <dir>]\n\nOptions:\n  --smoke    headless flow without opening a window (F5-02/registry + F5-08 full flow).\n  --root <dir>  resources root directory (default: examples).";

/// Mirrors the log buffer behind the port so `main` can inspect it after the
/// simulation without reading the return value of `step_tick`.
struct Port(std::rc::Rc<std::cell::RefCell<instinct_gui::LogBuffer>>);

impl WorldOutput for Port {
    fn emit(&mut self, event: &instinct_core::event::Event) {
        self.0.borrow_mut().push_event(event);
    }
}

/// Headless smoke flow: build a minimal world from `examples`, connect a
/// `LogBuffer` through `sim.world.set_output(...)`, run `step_tick` N times
/// and print what arrived through the port. No window is opened.
fn run_smoke() -> Result<(), String> {
    let resources = resources::load_resources(&examples_root());

    for diagnostic in &resources.diagnostics {
        eprintln!("{diagnostic}");
    }

    let map = resources
        .maps
        .iter()
        .find(|m| m.name == "Plain with rocks")
        .ok_or("map 'Plain with rocks' not found in examples")?;
    let species = resources
        .creatures
        .iter()
        .find(|s| s.name == "Talker")
        .ok_or("species 'Talker' not found in examples")?;

    let mut world = world::World::from_map(map, &resources.terrains, &resources.objects);
    let species = std::rc::Rc::new(species.clone());
    world
        .place_creature(&species, 1, 0)
        .map_err(|d| format!("cannot place Talker: {d}"))?;

    let mut sim = Simulation::new(world);
    let buffer = std::rc::Rc::new(std::cell::RefCell::new(
        instinct_gui::LogBuffer::with_capacity(1000),
    ));
    sim.world.set_output(Box::new(Port(Rc::clone(&buffer))));
    sim.set_rng_seed(7);

    for _ in 0..5 {
        let _ = sim.step_tick(); // return value deliberately ignored
    }

    let borrow = buffer.borrow();
    let entries = borrow.entries();
    if entries.is_empty() {
        return Err("no events arrived through the port".to_string());
    }
    for entry in entries.iter() {
        println!("{}", entry.line);
    }
    println!("== {} entries through the port ==", entries.len());
    Ok(())
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut root = examples_root();
    let mut smoke = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--smoke" => smoke = true,
            "--root" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("{USAGE}");
                    exit(2);
                }
                root = PathBuf::from(&args[i]);
            }
            _ => {
                eprintln!("{USAGE}");
                exit(2);
            }
        }
        i += 1;
    }

    if smoke {
        if let Err(message) = run_smoke() {
            eprintln!("instinct-gui: --smoke failed: {message}");
            exit(1);
        }
        return Ok(());
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
        Box::new(move |_cc| {
            Ok(Box::new(SpikeApp {
                state: AppState {
                    selected: None,
                    session: Some(LoadedSession::new(&root)),
                    prep_state: None,
                    prep_selected_species: 0,
                    time: Default::default(),
                    sim: None,
                    log_buffer: None,
                },
                last_time: None,
            }))
        }),
    )
}
