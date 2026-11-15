mod args;
mod check;
mod command;
mod output;
mod run;

use std::process::exit;

use crate::command::build_command;

/// Usage text of the program. Describes the `run` and `check` subcommands.
pub(crate) const USAGE: &str = "Usage: instinct-cli <command> [arguments]\n\nCommands:\n  run --root <dir> --map <name> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Species>@<x>,<y>]\n  check <file.ins>\n\nRun `instinct-cli --help` to view this help.";

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
