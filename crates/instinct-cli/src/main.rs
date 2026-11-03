use std::process::exit;

/// Texto de uso del programa. Describe los subcomandos `run` y `check`
/// (implementados en F0-02 y F0-06; hasta entonces se tratan como desconocidos).
const USAGE: &str = "Uso: instinct-cli <comando> [argumentos]\n\nComandos:\n  run --root <dir> --map <nombre> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Especie>@<x>,<y>]\n  check <archivo.ins>\n\nEjecute `instinct-cli --help` para ver esta ayuda.";

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

    // Cualquier otra cosa: subcomando desconocido (run y check aún no implementados).
    eprintln!("{USAGE}");
    eprintln!("error: subcomando o argumentos no reconocidos");
    exit(2);
}
