use std::process::{Command, Stdio};

/// Bank resources root directory: `examples/`, relative to this crate's
/// manifest.
pub fn bank_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
}

/// Locates the integration test binary.
pub fn instinct_cli_binary() -> std::path::PathBuf {
    std::env::var("BIN_INSTINCT_CLI")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(env!("CARGO_BIN_EXE_instinct-cli")).to_path_buf())
}

/// Executes the real instinct-cli binary with the given args.
pub fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(instinct_cli_binary())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("could not run the instinct-cli binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

/// Trampas resources root directory: `examples/traps/`, relative to this
/// crate's manifest.
pub fn traps_dir() -> String {
    format!("{}/../../examples/traps", env!("CARGO_MANIFEST_DIR"))
}

/// ASCII grid rows printed after the `== tick N ==` header,
/// up to the next tick header (or end of output).
pub fn rows_after_tick(stdout: &str, tick: u64) -> Vec<String> {
    let marker = format!("== tick {tick} ==");
    let mut rows: Vec<String> = Vec::new();
    let mut found = false;
    for line in stdout.lines() {
        if line == marker {
            found = true;
            continue;
        }
        if found {
            if line.starts_with("== tick ") || line.starts_with("[tick ") {
                break;
            }
            rows.push(line.to_string());
        }
    }
    rows
}
