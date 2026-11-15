use crate::args::{parse_run, RunArgs};
use crate::{check, run};

/// Common operation of every subcommand: returns the exit code.
pub(crate) trait Command {
    fn execute(&self) -> i32;
}

/// `run --root <dir> --map <nombre> [--seed <n>] [--ticks <n>] [--every <n>] [--place <Especie>@<x>,<y>]`
pub(crate) struct Run {
    args: RunArgs,
}

impl Command for Run {
    fn execute(&self) -> i32 {
        run::execute_run(&self.args)
    }
}

/// `check <archivo.ins>`
pub(crate) struct Check {
    path: String,
}

impl Command for Check {
    fn execute(&self) -> i32 {
        check::execute_check(&self.path)
    }
}

/// `--help` / `-h`: prints usage to stdout and exits 0.
pub(crate) struct Help;

impl Command for Help {
    fn execute(&self) -> i32 {
        println!("{}", crate::USAGE);
        0
    }
}

/// Builds the command object from the arguments after the program name.
/// Rejects unknown subcommands and unknown flags (C1).
pub(crate) fn build_command(args: &[String]) -> Result<Box<dyn Command>, String> {
    if args.is_empty() {
        return Err("expected a subcommand: run or check".to_string());
    }
    match args[0].as_str() {
        "run" => {
            let run_args = parse_run(&args[1..])?;
            Ok(Box::new(Run { args: run_args }))
        }
        "check" => {
            if args.len() != 2 {
                return Err("the 'check' subcommand requires a .ins file".to_string());
            }
            Ok(Box::new(Check {
                path: args[1].clone(),
            }))
        }
        "--help" | "-h" => {
            if args.len() != 1 {
                return Err("--help does not accept arguments".to_string());
            }
            Ok(Box::new(Help))
        }
        _ => Err(format!(
            "unrecognized subcommand or argument: '{}'",
            args[0]
        )),
    }
}
