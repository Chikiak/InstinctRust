mod args;
mod check;
mod command;
mod output;
mod run;

use std::process::exit;

use crate::command::build_command;

/// Usage text of the program. Describes the `run` and `check` subcommands.
pub(crate) const USAGE: &str = "Uso: instinct-cli <comando> [argumentos]\n\nComandos:\n  run --root <dir> --map <nombre> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Especie>@<x>,<y>]\n  check <archivo.ins>\n\nEjecute `instinct-cli --help` para ver esta ayuda.";

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // No arguments: print usage to stderr and exit with code 2.
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
