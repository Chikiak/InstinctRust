//! Integration tests for the `check` subcommand (F0-06, AC7).

use std::process::{Command, Stdio};

/// Example resources root directory, relative to this crate's manifest.
fn examples_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
}

/// Fixtures directory for invalid `.ins` files.
fn fixtures_dir() -> String {
    format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"))
}

/// Locates the integration test binary.
fn instinct_cli_binary() -> std::path::PathBuf {
    std::env::var("BIN_INSTINCT_CLI")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(env!("CARGO_BIN_EXE_instinct-cli")).to_path_buf())
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
fn test_cli_check_valid_exits_0() {
    let (code, stdout, stderr) =
        run_cli(&["check", &format!("{}/creatures/walker.ins", examples_dir())]);
    assert_eq!(code, 0, "valid file should exit 0, stderr: {stderr}");
    assert_eq!(
        stdout.trim(),
        "OK",
        "stdout should be exactly 'OK', got: {stdout}"
    );
    assert!(stderr.is_empty(), "stderr should be empty, got: {stderr}");
}

#[test]
fn test_cli_check_invalid_exits_1() {
    let (code, stdout, stderr) = run_cli(&["check", &format!("{}/bad_header.ins", fixtures_dir())]);
    assert_eq!(code, 1, "invalid file should exit 1");
    assert!(stdout.is_empty(), "stdout should be empty, got: {stdout}");
    assert!(
        !stderr.is_empty(),
        "stderr should contain diagnostics, got: {stderr}"
    );
    // Diagnostic form: archivo:línea: mensaje
    assert!(
        stderr.contains("missing required header key 'faction'")
            || stderr.contains("missing required header key 'lifespan'"),
        "stderr should name the missing key, got: {stderr}"
    );
}

#[test]
fn test_cli_check_bad_label() {
    let (code, _, stderr) = run_cli(&["check", &format!("{}/bad_label.ins", fixtures_dir())]);
    assert_eq!(code, 1, "bad label should exit 1");
    assert!(
        stderr.contains("goto to unknown label"),
        "stderr should name the unknown label, got: {stderr}"
    );
}

#[test]
fn test_cli_check_bad_action() {
    let (code, _, stderr) = run_cli(&["check", &format!("{}/bad_action.ins", fixtures_dir())]);
    assert_eq!(code, 1, "bad action should exit 1");
    assert!(
        stderr.contains("fly"),
        "stderr should name the unknown action, got: {stderr}"
    );
}

#[test]
fn test_cli_check_nonexistent_file() {
    let (code, _, stderr) = run_cli(&["check", "/no/such/file.ins"]);
    assert_eq!(code, 1, "nonexistent file should exit 1");
    assert!(!stderr.is_empty(), "stderr should report the error");
}
