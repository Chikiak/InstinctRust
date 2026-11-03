use std::process::exit;

use instinct_core::resources;
use instinct_core::world;

/// Usage text of the program. Describes the `run` and `check` subcommands
/// (`check` is implemented in F0-06; until then it is treated as unknown).
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
        "check" => {
            // check not yet implemented (F0-06).
            eprintln!("{USAGE}");
            eprintln!("error: subcomando 'check' aún no implementado");
            exit(2);
        }
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

    let world = world::World::from_map(map, &resources.terrains, &resources.objects);
    for d in &world.diagnostics {
        eprintln!("{d}");
    }

    // --seed, --ticks, --every, and --place are read and validated, but not
    // yet used in the simulation (F0-07 onwards).
    let _ = (&parsed.seed, &parsed.ticks, &parsed.every, &parsed.places);

    println!("== tick 0 ==");
    print!("{}", world.draw(&resources.terrains, &resources.objects));
    0
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
                every = Some(
                    v.parse::<i64>()
                        .map_err(|_| format!("--every debe ser un entero, obtuvo '{v}'"))?,
                );
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
