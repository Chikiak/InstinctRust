use super::{examples_dir, run_cli};

// C4 suite: `run` argument validation cases.

#[test]
fn test_cli_missing_map_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&["run", "--root", &root, "--ticks", "0"]);
    assert_eq!(
        code, 2,
        "run without --map must exit with 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must report the lack of --map, got: {stderr}"
    );
    assert!(
        stderr.contains("--map"),
        "stderr must name --map, got: {stderr}"
    );
}

#[test]
fn test_cli_negative_ticks_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "-3",
    ]);
    assert_eq!(
        code, 2,
        "--ticks negative must exit with 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must report the error, got: {stderr}"
    );
    assert!(
        stderr.contains("--ticks") || stderr.contains("ticks"),
        "stderr must name --ticks, got: {stderr}"
    );
}

#[test]
fn test_cli_missing_root_errors() {
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        "no/existe/este/directorio",
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
    ]);
    assert_eq!(
        code, 2,
        "--root nonexistent must exit with 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must report the error, got: {stderr}"
    );
    assert!(
        stderr.contains("no/existe/este/directorio"),
        "stderr must name the directory, got: {stderr}"
    );
}

#[test]
fn test_cli_every_zero_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "2",
        "--every",
        "0",
    ]);
    assert_eq!(code, 2, "--every 0 must exit with 2, stderr: {stderr}");
    assert!(
        !stderr.is_empty(),
        "stderr must report the error, got: {stderr}"
    );
}

#[test]
fn test_cli_malformed_place_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--place",
        "formato-mal",
    ]);
    assert_eq!(
        code, 1,
        "--place malformed must exit with 1, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must report the error, got: {stderr}"
    );
}

#[test]
fn test_cli_unknown_species_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--place",
        "NoExiste@0,0",
    ]);
    assert_eq!(
        code, 1,
        "unknown species must exit with 1, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must report the error, got: {stderr}"
    );
    assert!(
        stderr.contains("NoExiste"),
        "stderr must name the species, got: {stderr}"
    );
}

#[test]
fn test_cli_place_out_of_bounds_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--place",
        "Walker@9999,9999",
    ]);
    assert_eq!(
        code, 1,
        "out of bounds placement must exit with 1, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr must report the error, got: {stderr}"
    );
}
