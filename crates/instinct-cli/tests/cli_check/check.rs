//! Integration tests for the `check` subcommand (F0-06, AC7).

use super::{examples_dir, fixtures_dir, run_cli};

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
    // Diagnostic form: file:line: message
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

#[test]
fn test_cli_check_wrong_arg_count_errors() {
    // check with no args -> exit 2 (incorrect usage).
    let (code, _, stderr) = run_cli(&["check"]);
    assert_eq!(
        code, 2,
        "check sin archivo debe salir con 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );

    // check with two files -> exit 2 (incorrect usage).
    let root = format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"));
    let (code, _, stderr) = run_cli(&[
        "check",
        &format!("{}/creatures/walker.ins", root),
        &format!("{}/creatures/walker.ins", root),
    ]);
    assert_eq!(
        code, 2,
        "check con dos archivos debe salir con 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr should report the error, got: {stderr}"
    );
}
