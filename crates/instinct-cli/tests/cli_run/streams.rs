use super::{examples_dir, run_cli};

#[test]
fn test_cli_streams_separated() {
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    // The simulation (map) goes to stdout.
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout must contain the map, got:\n{stdout}"
    );
    // Load diagnostics go to stderr; for a valid map it is empty.
    assert!(
        stderr.is_empty(),
        "stderr must be empty for a valid run, got: {stderr}"
    );
}

// AC1: unknown flag (delegated to cli_usage.rs, but verifies here that it is
// not silently ignored).
#[test]
fn test_cli_unknown_flag_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--bandera-inexistente",
    ]);
    assert_eq!(code, 2, "unknown flag must exit with 2, stderr: {stderr}");
    assert!(
        !stderr.is_empty(),
        "stderr must report the error, got: {stderr}"
    );
}
