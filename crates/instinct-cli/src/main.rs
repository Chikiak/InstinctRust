use std::path::Path;
use std::process::exit;
use std::rc::Rc;

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::WorldOutput;
use instinct_core::perception::PerceptionCatalog;
use instinct_core::resources;
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world;

/// Usage text of the program. Describes the `run` and `check` subcommands.
const USAGE: &str = "Uso: instinct-cli <comando> [argumentos]\n\nComandos:\n  run --root <dir> --map <nombre> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Especie>@<x>,<y>]\n  check <archivo.ins>\n\nEjecute `instinct-cli --help` para ver esta ayuda.";

/// Common operation of every subcommand: returns the exit code.
trait Command {
    fn execute(&self) -> i32;
}

/// `run --root <dir> --map <nombre> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Especie>@<x>,<y>]`
struct Run {
    args: RunArgs,
}

impl Command for Run {
    fn execute(&self) -> i32 {
        execute_run(&self.args)
    }
}

/// `check <archivo.ins>`
struct Check {
    path: String,
}

impl Command for Check {
    fn execute(&self) -> i32 {
        execute_check(&self.path)
    }
}

/// `--help` / `-h`: prints usage to stdout and exits 0.
struct Help;

impl Command for Help {
    fn execute(&self) -> i32 {
        println!("{USAGE}");
        0
    }
}

/// Arguments of the `run` subcommand, fully validated before construction.
struct RunArgs {
    root: String,
    map_name: String,
    seed: u64,
    ticks: i64,
    every: i64,
    places: Vec<String>,
}

/// A parsed creature placement: species name and map coordinates.
struct Placement {
    species: String,
    x: usize,
    y: usize,
}

/// Puerto de salida del mundo para la CLI: imprime cada evento a stdout.
/// Los diagnósticos de carga siguen yendo a stderr por caminos separados.
struct CliOutput;

impl WorldOutput for CliOutput {
    fn emit(&mut self, event: &instinct_core::event::Event) {
        println!("{event}");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // No arguments: usage to stderr, exit 2.
    if args.len() == 1 {
        eprintln!("{USAGE}");
        exit(2);
    }

    let command = match build_command(&args[1..]) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("{USAGE}");
            eprintln!("error: {e}");
            exit(2);
        }
    };

    exit(command.execute());
}

/// Builds the command object from the arguments after the program name.
/// Rejects unknown subcommands and unknown flags (C1).
fn build_command(args: &[String]) -> Result<Box<dyn Command>, String> {
    if args.is_empty() {
        return Err("se espera un subcomando: run o check".to_string());
    }
    match args[0].as_str() {
        "run" => {
            let run_args = parse_run(&args[1..])?;
            Ok(Box::new(Run { args: run_args }))
        }
        "check" => {
            if args.len() != 2 {
                return Err("el subcomando 'check' requiere un archivo .ins".to_string());
            }
            Ok(Box::new(Check {
                path: args[1].clone(),
            }))
        }
        "--help" | "-h" => {
            if args.len() != 1 {
                return Err("--help no acepta argumentos".to_string());
            }
            Ok(Box::new(Help))
        }
        _ => Err(format!(
            "subcomando o argumento no reconocido: '{}'",
            args[0]
        )),
    }
}

/// Executes the `run` subcommand. Returns the exit code.
/// The map and tick events go to stdout; diagnostics go to stderr (C5).
fn execute_run(args: &RunArgs) -> i32 {
    let root = Path::new(&args.root);
    let resources = resources::load_resources(root);

    // Load diagnostics are reported to stderr.
    for d in &resources.diagnostics {
        eprintln!("{d}");
    }

    // Find the map by name.
    let map = resources.maps.iter().find(|m| m.name == args.map_name);
    let map = match map {
        Some(m) => m,
        None => {
            eprintln!("mapa no encontrado: '{}'", args.map_name);
            if !resources.maps.is_empty() {
                eprint!("mapas disponibles: ");
                for (i, m) in resources.maps.iter().enumerate() {
                    if i > 0 {
                        eprint!(", ");
                    }
                    eprint!("{}", m.name);
                }
                eprintln!();
            }
            return 1;
        }
    };

    if resources.terrains.is_empty() {
        eprintln!("no hay terrenos cargados para dibujar el mapa");
        return 1;
    }

    // Parse every --place value. A malformed placement or an unknown species
    // aborts with exit code 1 before simulating.
    let mut placements: Vec<(Placement, Rc<Species>)> = Vec::new();
    for place_str in &args.places {
        let placement = match parse_place(place_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{e}");
                return 1;
            }
        };
        let species = match resources
            .creatures
            .iter()
            .find(|s| s.name == placement.species)
        {
            Some(s) => Rc::new(s.clone()),
            None => {
                eprintln!("especie desconocida: '{}'", placement.species);
                return 1;
            }
        };
        placements.push((placement, species));
    }

    // Build the world and place creatures. Cada criatura apunta al mismo
    // `Rc<Species>` que el mundo registra en su catálogo (flyweight).
    let mut world = world::World::from_map(map, &resources.terrains, &resources.objects);
    for d in &world.diagnostics {
        eprintln!("{d}");
    }

    for (placement, species) in &placements {
        match world.place_creature(species, placement.x, placement.y) {
            Ok(_) => {}
            Err(d) => {
                eprintln!("{d}");
                return 1;
            }
        }
    }

    // Create the simulation and seed its generator. El mundo ya es el único
    // dueño de los catálogos de terrenos y objetos.
    let mut sim = Simulation::new(world);
    // Conecta el puerto de salida: los eventos van a stdout vía CliOutput.
    sim.world.set_output(Box::new(CliOutput));
    sim.set_rng_seed(args.seed);

    let ticks = args.ticks;
    let every = args.every;

    // Map at tick 0 (initial state).
    println!("== tick 0 ==");
    match sim.world().draw() {
        Ok(drawing) => print!("{drawing}"),
        Err(d) => {
            eprintln!("{d}");
            return 1;
        }
    }

    for _ in 0..ticks {
        // step_tick avanza el tick y los eventos se imprimen vía el puerto
        // de salida (CliOutput). Se descarta el valor de retorno; la API
        // sigue devolviendo Vec<Event> para compatibilidad.
        let _ = sim.step_tick();

        let next_tick = sim.tick();

        // Print the map when the tick counter hits a multiple of `--every`
        // or the final tick.
        let should_print =
            every > 0 && (next_tick.is_multiple_of(every as u64) || next_tick == ticks as u64);
        if should_print {
            println!("== tick {next_tick} ==");
            match sim.world().draw() {
                Ok(drawing) => print!("{drawing}"),
                Err(d) => {
                    eprintln!("{d}");
                    return 1;
                }
            }
        }
    }

    0
}

/// Parses a `--place` value of the form `Especie@x,y`.
fn parse_place(value: &str) -> Result<Placement, String> {
    let at_pos = value.find('@').ok_or_else(|| {
        format!("'--place' tiene formato invalido: '{value}', se esperaba Especie@x,y")
    })?;
    let species = value[..at_pos].to_string();
    if species.is_empty() {
        return Err(format!(
            "'--place' tiene formato invalido: '{value}', falta el nombre de la especie"
        ));
    }
    let coords = &value[at_pos + 1..];
    let comma_pos = coords.find(',').ok_or_else(|| {
        format!("'--place' tiene formato invalido: '{value}', se esperaba Especie@x,y")
    })?;
    let x_str = &coords[..comma_pos];
    let y_str = &coords[comma_pos + 1..];
    let x: usize = x_str
        .parse()
        .map_err(|_| format!("'--place' coordenada x invalida: '{x_str}' en '{value}'"))?;
    let y: usize = y_str
        .parse()
        .map_err(|_| format!("'--place' coordenada y invalida: '{y_str}' en '{value}'"))?;
    Ok(Placement { species, x, y })
}

/// Builds and validates the arguments of the `run` subcommand (after the `run`
/// name). Rejects unknown flags (C1), validates `--ticks >= 0` (C2),
/// `--every >= 1` and that `--root` exists (C3).
fn parse_run(args: &[String]) -> Result<RunArgs, String> {
    let mut root: Option<String> = None;
    let mut map_name: Option<String> = None;
    let mut seed: Option<u64> = None;
    let mut ticks: Option<i64> = None;
    let mut every: Option<i64> = None;
    let mut places: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let token = &args[i];
        match token.as_str() {
            "--root" => {
                root = Some(next_value(args, &mut i, "--root")?);
            }
            "--map" => {
                map_name = Some(next_value(args, &mut i, "--map")?);
            }
            "--seed" => {
                let v = next_value(args, &mut i, "--seed")?;
                seed = Some(
                    v.parse::<u64>()
                        .map_err(|_| format!("--seed debe ser un entero, obtuvo '{v}'"))?,
                );
            }
            "--ticks" => {
                let v = next_value(args, &mut i, "--ticks")?;
                let parsed = v
                    .parse::<i64>()
                    .map_err(|_| format!("--ticks debe ser un entero, obtuvo '{v}'"))?;
                if parsed < 0 {
                    return Err(format!(
                        "--ticks debe ser mayor o igual a 0, obtuvo {parsed}"
                    ));
                }
                ticks = Some(parsed);
            }
            "--every" => {
                let v = next_value(args, &mut i, "--every")?;
                let parsed = v
                    .parse::<i64>()
                    .map_err(|_| format!("--every debe ser un entero, obtuvo '{v}'"))?;
                if parsed < 1 {
                    return Err(format!(
                        "--every debe ser mayor o igual a 1, obtuvo {parsed}"
                    ));
                }
                every = Some(parsed);
            }
            "--place" => {
                places.push(next_value(args, &mut i, "--place")?);
            }
            _ => {
                return Err(format!("flag desconocido: '{token}'"));
            }
        }
    }

    let root = root.unwrap_or_else(|| "examples".to_string());

    // C3: validate that the root directory exists, whether explicit or default.
    if !Path::new(&root).is_dir() {
        return Err(format!("el directorio --root no existe: '{root}'"));
    }

    let map_name = map_name.ok_or_else(|| "falta el argumento obligatorio --map".to_string())?;

    Ok(RunArgs {
        root,
        map_name,
        seed: seed.unwrap_or(0),
        ticks: ticks.unwrap_or(0),
        every: every.unwrap_or(1),
        places,
    })
}

/// Reads the value following a flag, advancing `i`.
fn next_value(args: &[String], i: &mut usize, flag: &str) -> Result<String, String> {
    let value = args
        .get(*i + 1)
        .ok_or_else(|| format!("{flag} requires a value"))?;
    *i += 2;
    Ok(value.clone())
}

/// Executes the `check` subcommand. Returns the exit code.
fn execute_check(path: &str) -> i32 {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{USAGE}");
            eprintln!("error: no se pudo leer el archivo '{path}': {e}");
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
