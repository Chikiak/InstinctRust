use std::process::{Command, Stdio};

/// Locates the binary for the integration test via the environment variable
/// Cargo provides: `BIN_INSTINCT_CLI` (set by `#[test]` with `#[path]` or by
/// the cargo test harness).
fn instinct_cli_binary() -> std::path::PathBuf {
    std::env::var("BIN_INSTINCT_CLI")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            // Fallback: look in target/debug (cargo test puts it there).
            std::path::Path::new(env!("CARGO_BIN_EXE_instinct-cli")).to_path_buf()
        })
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
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

#[test]
fn test_cli_without_args_exits_2() {
    let (code, stdout, stderr) = run_cli(&[]);
    assert_eq!(code, 2, "no arguments should exit with 2");
    assert!(stdout.is_empty(), "no arguments should not write to stdout");
    assert!(
        !stderr.is_empty(),
        "no arguments should write usage to stderr"
    );
    assert!(
        stderr.contains("Usage:"),
        "stderr should contain the usage text, got: {stderr}"
    );
}

#[test]
fn test_cli_help_exits_0() {
    let (code, stdout, stderr) = run_cli(&["--help"]);
    assert_eq!(code, 0, "--help should exit with 0");
    assert!(!stdout.is_empty(), "--help should write usage to stdout");
    assert!(stderr.is_empty(), "--help should not write to stderr");
    assert!(
        stdout.contains("Usage:"),
        "stdout should contain the usage text, got: {stdout}"
    );
    assert!(
        stdout.contains("run") && stdout.contains("check"),
        "usage should mention run and check"
    );
}

#[test]
fn test_cli_unknown_subcommand_exits_2() {
    let (code, _, stderr) = run_cli(&["inexistente"]);
    assert_eq!(code, 2, "unimplemented subcommand should exit with 2");
    assert!(
        !stderr.is_empty(),
        "unknown subcommand should write a message to stderr"
    );
}
