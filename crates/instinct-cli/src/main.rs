use std::process::exit;

use instinct_core::resources;
use instinct_core::world;

/// Texto de uso del programa. Describe los subcomandos `run` y `check`
/// (`check` se implementa en F0-06; hasta entonces se trata como desconocido).
const USAGE: &str = "Uso: instinct-cli <comando> [argumentos]\n\nComandos:\n  run --root <dir> --map <nombre> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Especie>@<x>,<y>]\n  check <archivo.ins>\n\nEjecute `instinct-cli --help` para ver esta ayuda.";

/// Argumentos del subcomando `run`.
struct RunArgs {
    root: String,
    mapa: String,
    seed: u64,
    ticks: i64,
    every: i64,
    places: Vec<String>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // Sin argumentos: uso por stderr, salida 2.
        eprintln!("{USAGE}");
        exit(2);
    }

    if args.len() == 2 && (args[1] == "--help" || args[1] == "-h") {
        // --help: uso por stdout, salida 0.
        println!("{USAGE}");
        exit(0);
    }

    match args[1].as_str() {
        "run" => exit(ejecutar_run(&args[2..])),
        "check" => {
            // check aún no implementado (F0-06).
            eprintln!("{USAGE}");
            eprintln!("error: subcomando 'check' aún no implementado");
            exit(2);
        }
        _ => {
            // Cualquier otra cosa: subcomando desconocido.
            eprintln!("{USAGE}");
            eprintln!("error: subcomando o argumentos no reconocidos");
            exit(2);
        }
    }
}

/// Ejecuta el subcomando `run`. Devuelve el código de salida.
fn ejecutar_run(args: &[String]) -> i32 {
    let parsed = match parsear_run(args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{USAGE}");
            eprintln!("error: {e}");
            return 2;
        }
    };

    let raiz = std::path::Path::new(&parsed.root);
    let recursos = resources::cargar_recursos(raiz);

    // Los diagnósticos de carga se reportan por stderr.
    for d in &recursos.diagnosticos {
        eprintln!("{d}");
    }

    // Buscar el mapa por nombre.
    let mapa = recursos.mapas.iter().find(|m| m.nombre == parsed.mapa);
    let mapa = match mapa {
        Some(m) => m,
        None => {
            eprintln!("mapa no encontrado: '{}'", parsed.mapa);
            if !recursos.mapas.is_empty() {
                eprint!("mapas disponibles: ");
                for (i, m) in recursos.mapas.iter().enumerate() {
                    if i > 0 {
                        eprint!(", ");
                    }
                    eprint!("{}", m.nombre);
                }
                eprintln!();
            }
            return 1;
        }
    };

    if recursos.terrenos.is_empty() {
        eprintln!("no hay terrenos cargados para dibujar el mapa");
        return 1;
    }

    let mundo = world::World::desde_mapa(mapa, &recursos.terrenos, &recursos.objetos);
    for d in &mundo.diagnosticos {
        eprintln!("{d}");
    }

    // --seed, --ticks, --every y --place se leen y validan, pero aún no
    // se usan en la simulación (F0-07 en adelante).
    let _ = (&parsed.seed, &parsed.ticks, &parsed.every, &parsed.places);

    println!("== tick 0 ==");
    print!("{}", mundo.dibujar(&recursos.terrenos, &recursos.objetos));
    0
}

/// Parsea los argumentos del subcomando `run` (tras el nombre `run`).
fn parsear_run(args: &[String]) -> Result<RunArgs, String> {
    let mut root: Option<String> = None;
    let mut mapa: Option<String> = None;
    let mut seed: Option<u64> = None;
    let mut ticks: Option<i64> = None;
    let mut every: Option<i64> = None;
    let mut places: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let token = &args[i];
        match token.as_str() {
            "--root" => {
                root = Some(valor_siguiente(args, &mut i, "--root")?);
            }
            "--map" => {
                mapa = Some(valor_siguiente(args, &mut i, "--map")?);
            }
            "--seed" => {
                let v = valor_siguiente(args, &mut i, "--seed")?;
                seed = Some(
                    v.parse::<u64>()
                        .map_err(|_| format!("--seed debe ser un entero, obtuvo '{v}'"))?,
                );
            }
            "--ticks" => {
                let v = valor_siguiente(args, &mut i, "--ticks")?;
                ticks = Some(
                    v.parse::<i64>()
                        .map_err(|_| format!("--ticks debe ser un entero, obtuvo '{v}'"))?,
                );
            }
            "--every" => {
                let v = valor_siguiente(args, &mut i, "--every")?;
                every = Some(
                    v.parse::<i64>()
                        .map_err(|_| format!("--every debe ser un entero, obtuvo '{v}'"))?,
                );
            }
            "--place" => {
                places.push(valor_siguiente(args, &mut i, "--place")?);
            }
            _ => {
                i += 1;
            }
        }
    }

    let root = root.unwrap_or_else(|| "examples".to_string());
    let mapa = mapa.ok_or_else(|| "falta el argumento obligatorio --map".to_string())?;

    Ok(RunArgs {
        root,
        mapa,
        seed: seed.unwrap_or(0),
        ticks: ticks.unwrap_or(0),
        every: every.unwrap_or(1),
        places,
    })
}

/// Lee el valor que sigue a una bandera, avanzando `i`.
fn valor_siguiente(args: &[String], i: &mut usize, bandera: &str) -> Result<String, String> {
    let valor = args
        .get(*i + 1)
        .ok_or_else(|| format!("{bandera} requiere un valor"))?;
    *i += 2;
    Ok(valor.clone())
}
