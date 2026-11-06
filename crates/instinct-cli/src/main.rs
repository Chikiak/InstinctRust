use std::process::exit;

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::perception::PerceptionCatalog;
use instinct_core::resources;
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world;

/// Usage text of the program. Describes the `run` and `check` subcommands.
const USAGE: &str = "Uso: instinct-cli <comando> [argumentos]\n\nComandos:\n  run --root <dir> --map <nombre> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Especie>@<x>,<y>]\n  check <archivo.ins>\n\nEjecute `instinct-cli --help` para ver esta ayuda.";

/// Arguments of the `run` subcommand.
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

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // No arguments: usage to stderr, exit 2.
        eprintln!("{USAGE}");
        exit(2);
    }

    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        // --help: usage to stdout, exit 0.
        println!("{USAGE}");
        exit(0);
    }

    match args[1].as_str() {
        "run" => exit(execute_run(&args[2..])),
        "check" => exit(execute_check(&args[2..])),
        _ => {
            // Anything else: unknown subcommand.
            eprintln!("{USAGE}");
            eprintln!("error: subcomando o argumentos no reconocidos");
            exit(2);
        }
    }
}

/// Executes the `run` subcommand. Returns the exit code.
fn execute_run(args: &[String]) -> i32 {
    let parsed = match parse_run(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{USAGE}");
            eprintln!("error: {e}");
            return 2;
        }
    };

    let root = std::path::Path::new(&parsed.root);
    let resources = resources::load_resources(root);

    // Load diagnostics are reported to stderr.
    for d in &resources.diagnostics {
        eprintln!("{d}");
    }

    // Find the map by name.
    let map = resources.maps.iter().find(|m| m.name == parsed.map_name);
    let map = match map {
        Some(m) => m,
        None => {
            eprintln!("mapa no encontrado: '{}'", parsed.map_name);
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
    let mut placements: Vec<(Placement, Species)> = Vec::new();
    for place_str in &parsed.places {
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
            Some(s) => s.clone(),
            None => {
                eprintln!("especie desconocida: '{}'", placement.species);
                return 1;
            }
        };
        placements.push((placement, species));
    }

    // Build the world and place creatures.
    let mut world = world::World::from_map(map, &resources.terrains, &resources.objects);
    for d in &world.diagnostics {
        eprintln!("{d}");
    }

    for (placement, species) in &placements {
        match world.place_creature(species.clone(), placement.x, placement.y) {
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
    sim.set_rng_seed(parsed.seed);

    let ticks = parsed.ticks;
    let every = parsed.every;

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
        let events = sim.step_tick();

        // Events from the tick that just ran (they carry the pre-increment
        // tick number) are printed after that tick's map.
        for event in &events {
            println!("{event}");
        }

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

/// Parses the arguments of the `run` subcommand (after the `run` name).
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
                ticks = Some(
                    v.parse::<i64>()
                        .map_err(|_| format!("--ticks debe ser un entero, obtuvo '{v}'"))?,
                );
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
                i += 1;
            }
        }
    }

    let root = root.unwrap_or_else(|| "examples".to_string());
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
fn execute_check(args: &[String]) -> i32 {
    if args.len() != 1 {
        eprintln!("{USAGE}");
        eprintln!("error: el subcomando 'check' requiere un archivo .ins");
        return 2;
    }

    let path = &args[0];
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
