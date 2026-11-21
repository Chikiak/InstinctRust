use instinct_core::event::WorldOutput;
use instinct_core::resources;
use instinct_core::sim::Simulation;
use instinct_gui::{app::session::LoadedSession, app::AppState, app::Msg, ui::SpikeApp};
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

/// Puerto auxiliar para tests / main (mismo patrón que app::log).
struct Port(std::rc::Rc<std::cell::RefCell<instinct_gui::app::log::LogBuffer>>);

impl WorldOutput for Port {
    fn emit(&mut self, event: &instinct_core::event::Event) {
        self.0.borrow_mut().push_event(event);
    }
}

/// Imprime el registro completo (líneas del log) y devuelve Ok/Err con causa.
fn print_log(
    label: &str,
    buffer: &std::rc::Rc<std::cell::RefCell<instinct_gui::app::log::LogBuffer>>,
) -> Result<(), String> {
    let borrow = buffer.borrow();
    let entries = borrow.entries();
    println!("=== {label} ({} entradas) ===", entries.len());
    for entry in entries.iter() {
        println!("{}", entry.line);
    }
    Ok(())
}

/// Headless smoke flow (F5-08): ejecuta el flujo 4.1 completo sin ventana.
/// exit 0 si todo verde; imprime causa y exit 1 ante cualquier fallo.
fn run_smoke() -> Result<(), String> {
    let resources = resources::load_resources(&examples_root());

    for diagnostic in &resources.diagnostics {
        eprintln!("{diagnostic}");
    }

    // a) Carga examples/ (load_resources) + sesión.
    let map = resources
        .maps
        .iter()
        .find(|m| m.name == "Plain with rocks")
        .ok_or("map 'Plain with rocks' not found in examples")?;

    // b) Entra en preparación con mapa por defecto ("Plain with rocks").
    let mut app = AppState {
        session: Some(LoadedSession::new(&examples_root())),
        ..Default::default()
    };
    app.update(Msg::PrepEnter(0));
    let prep = app
        .prep_state
        .as_ref()
        .ok_or("no se pudo entrar en preparación")?;
    if prep.recipe.map_name != "Plain with rocks" {
        return Err(format!(
            "mapa de preparación inesperado: {}",
            prep.recipe.map_name
        ));
    }

    // c) Coloca Dwarf@1,0, Ent@5,1, Hobbit@10,0, Uruk@12,3.
    // Resolver índices de especies antes de cualquier mutación de `app` para
    // evitar borrados en conflicto con `app.update`.
    let species_names = app
        .session
        .as_ref()
        .unwrap()
        .species_names()
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>();
    let idx_of = |name: &str| {
        species_names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| format!("especie '{name}' no encontrada"))
    };
    let dwarf_idx = idx_of("Dwarf")?;
    let ent_idx = idx_of("Ent")?;
    let hobbit_idx = idx_of("Hobbit")?;
    let uruk_idx = idx_of("Uruk")?;
    app.update(Msg::PrepPlace(dwarf_idx, 1, 0));
    app.update(Msg::PrepPlace(ent_idx, 5, 1));
    app.update(Msg::PrepPlace(hobbit_idx, 10, 0));
    app.update(Msg::PrepPlace(uruk_idx, 12, 3));
    let prep = app
        .prep_state
        .as_ref()
        .ok_or("prep_state perdido tras colocar")?;
    if prep.placed_count() != 4 {
        return Err(format!(
            "se esperaban 4 colocaciones, hay {}",
            prep.placed_count()
        ));
    }

    // Estado inicial (tick 0, mismas criaturas colocadas) para comparar tras reinicio.
    let initial_creatures: Vec<(String, usize, usize)> = prep
        .creatures_in_prep()
        .iter()
        .map(|(_, name, x, y)| (name.clone(), *x, *y))
        .collect();
    let initial_seed = prep.recipe.seed;

    // d) Play con las 3 velocidades (T2, T10, T30).
    app.update(Msg::TickSetSpeed(instinct_gui::app::time::Speed::T2));
    app.update(Msg::TickPlay);
    app.update(Msg::TickSetSpeed(instinct_gui::app::time::Speed::T10));
    app.update(Msg::TickSetSpeed(instinct_gui::app::time::Speed::T30));
    if !matches!(
        app.time.state,
        instinct_gui::app::time::RunState::Running(_)
    ) {
        return Err("la simulación no está Running tras play".to_string());
    }

    // e) Pausa.
    app.update(Msg::TickPause);
    if !matches!(app.time.state, instinct_gui::app::time::RunState::Paused(_)) {
        return Err("la simulación no está Paused tras pausa".to_string());
    }

    // f) step_once (1 tick).
    app.update(Msg::TickStep);
    if app.sim.is_none() {
        return Err("sim no iniciada tras step_once".to_string());
    }
    if app.sim.as_ref().unwrap().world().tick() != 1 {
        return Err(format!(
            "se esperaba tick 1 tras step_once, es {}",
            app.sim.as_ref().unwrap().world().tick()
        ));
    }

    // g) restart().
    app.update(Msg::Restart);
    let prep2 = app
        .prep_state
        .as_ref()
        .ok_or("prep_state perdido tras restart")?;
    if app.sim.is_some() {
        return Err("sim debe ser None tras restart".to_string());
    }
    if !matches!(app.time.state, instinct_gui::app::time::RunState::Idle) {
        return Err("time debe estar Idle tras restart".to_string());
    }
    if app.time.tick_count() != 0 {
        return Err(format!(
            "time.tick debe ser 0 tras restart, es {}",
            app.time.tick_count()
        ));
    }

    // h) Verifica receta recreada produce mismo estado inicial (mismo tick 0, mismas criaturas colocadas).
    if prep2.recipe.map_name != "Plain with rocks" {
        return Err(format!(
            "mapa tras restart inesperado: {}",
            prep2.recipe.map_name
        ));
    }
    if prep2.placed_count() != 4 {
        return Err(format!(
            "tras restart se esperaban 4 colocaciones, hay {}",
            prep2.placed_count()
        ));
    }
    let recreated: Vec<(String, usize, usize)> = prep2
        .creatures_in_prep()
        .iter()
        .map(|(_, name, x, y)| (name.clone(), *x, *y))
        .collect();
    if recreated != initial_creatures {
        return Err(format!(
            "colocaciones tras restart difieren del inicial: {recreated:?} != {initial_creatures:?}"
        ));
    }
    if prep2.recipe.seed != initial_seed {
        return Err("semilla no preservada tras restart".to_string());
    }

    // i) Puerta de determinismo: 5 ticks con seed 7 vía GUI == 5× step_tick directo seed 7
    //    sobre la misma colocación.
    let gui_events = run_gui_5_ticks_seed7()?;
    let direct_events = run_direct_5_ticks_seed7(app.session.as_ref().unwrap(), map)?;
    if gui_events != direct_events {
        return Err(format!(
            "determinismo roto: GUI {gui_events:?} != directo {direct_events:?}"
        ));
    }

    // Imprime registro completo y sale exit 0.
    let buffer = app.log_buffer.as_ref().ok_or("log_buffer ausente")?;
    print_log("registro flujo 4.1 (--smoke)", buffer)?;
    println!("== --smoke OK: flujo 4.1 completo verificado ==");
    Ok(())
}

/// Ejecuta 5 ticks vía GUI (AppState) con seed 7 sobre la colocación del
/// baseline y devuelve las líneas de registro (eventos).
fn run_gui_5_ticks_seed7() -> Result<Vec<String>, String> {
    // Construir AppState desde cero: entrar en preparación, colocar las 4
    // criaturas del baseline, play (start_simulation usará recipe.seed = 7) y
    // avanzar 5 ticks (T2 = 500 ms/tick → 2500 ms).
    let mut app = AppState {
        session: Some(LoadedSession::new(&examples_root())),
        ..Default::default()
    };
    app.update(Msg::PrepEnter(0));
    // Forzar semilla 7 para la puerta de determinismo (PrepEnter hardcodea 0).
    app.prep_state.as_mut().unwrap().recipe.seed = 7;
    // Resolver índices de especies antes de mutar `app`.
    let species_names = app
        .session
        .as_ref()
        .unwrap()
        .species_names()
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>();
    let idx_of = |name: &str| {
        species_names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| format!("especie '{name}' no encontrada"))
    };
    for (name, x, y) in [
        ("Dwarf", 1usize, 0usize),
        ("Ent", 5, 1),
        ("Hobbit", 10, 0),
        ("Uruk", 12, 3),
    ] {
        let idx = idx_of(name)?;
        app.update(Msg::PrepPlace(idx, x, y));
    }
    app.update(Msg::TickPlay);
    app.update(Msg::TickAdvance(
        5 * instinct_gui::app::time::Speed::T2.ms_per_tick(),
    )); // 5 ticks

    let buffer = app
        .log_buffer
        .as_ref()
        .ok_or("log_buffer ausente en GUI determinismo")?;
    let entries = buffer.borrow().entries().to_vec();
    Ok(entries.into_iter().map(|e| e.line).collect())
}

/// Ejecuta 5× Simulation::step_tick directo con seed 7 sobre la misma colocación
/// y devuelve las líneas de registro (eventos).
fn run_direct_5_ticks_seed7(
    session: &LoadedSession,
    map: &instinct_core::resources::Map,
) -> Result<Vec<String>, String> {
    let mut world = instinct_core::world::World::from_map(
        map,
        &session.resources().terrains,
        &session.resources().objects,
    );
    for (name, x, y) in [
        ("Dwarf", 1, 0),
        ("Ent", 5, 1),
        ("Hobbit", 10, 0),
        ("Uruk", 12, 3),
    ] {
        let species = session
            .species()
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("especie '{name}' no encontrada"))?;
        let species = Rc::new(species.clone());
        world
            .place_creature(&species, x, y)
            .map_err(|d| format!("no se pudo colocar {name}: {d}"))?;
    }
    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);
    let buffer = Rc::new(std::cell::RefCell::new(
        instinct_gui::app::log::LogBuffer::with_capacity(1000),
    ));
    sim.world.set_output(Box::new(Port(Rc::clone(&buffer))));
    for _ in 0..5 {
        let _ = sim.step_tick();
    }
    let entries = buffer.borrow().entries().to_vec();
    Ok(entries.into_iter().map(|e| e.line).collect())
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
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 760.0]),
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
                    last_events: Vec::new(),
                },
                last_time: None,
            }))
        }),
    )
}
