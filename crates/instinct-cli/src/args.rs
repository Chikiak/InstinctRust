use std::path::Path;

/// Arguments of the `run` subcommand, fully validated before construction.
pub(crate) struct RunArgs {
    pub(crate) root: String,
    pub(crate) map_name: String,
    pub(crate) seed: u64,
    pub(crate) ticks: i64,
    pub(crate) every: i64,
    pub(crate) places: Vec<String>,
}

/// A parsed creature placement: species name and map coordinates.
pub(crate) struct Placement {
    pub(crate) species: String,
    pub(crate) x: usize,
    pub(crate) y: usize,
}

/// Parses a `--place` value of the form `Especie@x,y`.
pub(crate) fn parse_place(value: &str) -> Result<Placement, String> {
    let at_pos = value
        .find('@')
        .ok_or_else(|| format!("'--place' has invalid format: '{value}', expected Species@x,y"))?;
    let species = value[..at_pos].to_string();
    if species.is_empty() {
        return Err(format!(
            "'--place' has invalid format: '{value}', missing species name"
        ));
    }
    let coords = &value[at_pos + 1..];
    let comma_pos = coords
        .find(',')
        .ok_or_else(|| format!("'--place' has invalid format: '{value}', expected Species@x,y"))?;
    let x_str = &coords[..comma_pos];
    let y_str = &coords[comma_pos + 1..];
    let x: usize = x_str
        .parse()
        .map_err(|_| format!("'--place' invalid x coordinate: '{x_str}' in '{value}'"))?;
    let y: usize = y_str
        .parse()
        .map_err(|_| format!("'--place' invalid y coordinate: '{y_str}' in '{value}'"))?;
    Ok(Placement { species, x, y })
}

/// Builds and validates the arguments of the `run` subcommand (after the `run`
/// name). Rejects unknown flags (C1), validates `--ticks >= 0` (C2),
/// `--every >= 1` and that `--root` exists (C3).
pub(crate) fn parse_run(args: &[String]) -> Result<RunArgs, String> {
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
                        .map_err(|_| format!("--seed must be an integer, got '{v}'"))?,
                );
            }
            "--ticks" => {
                let v = next_value(args, &mut i, "--ticks")?;
                let parsed = v
                    .parse::<i64>()
                    .map_err(|_| format!("--ticks must be an integer, got '{v}'"))?;
                if parsed < 0 {
                    return Err(format!(
                        "--ticks must be greater than or equal to 0, got {parsed}"
                    ));
                }
                ticks = Some(parsed);
            }
            "--every" => {
                let v = next_value(args, &mut i, "--every")?;
                let parsed = v
                    .parse::<i64>()
                    .map_err(|_| format!("--every must be an integer, got '{v}'"))?;
                if parsed < 1 {
                    return Err(format!(
                        "--every must be greater than or equal to 1, got {parsed}"
                    ));
                }
                every = Some(parsed);
            }
            "--place" => {
                places.push(next_value(args, &mut i, "--place")?);
            }
            _ => {
                return Err(format!("unknown flag: '{token}'"));
            }
        }
    }

    let root = root.unwrap_or_else(|| "examples".to_string());

    // C3: validate that the root directory exists, whether explicit or default.
    if !Path::new(&root).is_dir() {
        return Err(format!("--root directory does not exist: '{root}'"));
    }

    let map_name = map_name.ok_or_else(|| "missing required argument --map".to_string())?;

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
