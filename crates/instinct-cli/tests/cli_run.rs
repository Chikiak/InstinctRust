use std::process::{Command, Stdio};

/// Example resources root directory, relative to this crate's manifest.
fn examples_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
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
fn test_cli_prints_map() {
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
    assert_eq!(code, 0, "exit code should be 0, stderr: {stderr}");
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout should contain '== tick 0 ==', got:\n{stdout}"
    );
    // The seven rows of the map, identical to the file.
    let expected = [
        "....................",
        "...##...............",
        "...##.....#.........",
        "..........#....###..",
        "...............#....",
        ".#..................",
        ".#........##........",
    ];
    for row in expected {
        assert!(
            stdout.contains(row),
            "stdout should contain the row '{row}', got:\n{stdout}"
        );
    }
    assert!(
        stderr.is_empty(),
        "stderr should be empty for a valid map, got: {stderr}"
    );
}

#[test]
fn test_cli_unknown_map_exits_1() {
    let root = examples_dir();
    let (code, _, stderr) =
        run_cli(&["run", "--root", &root, "--map", "No existe", "--ticks", "0"]);
    assert_eq!(code, 1, "nonexistent map should exit with 1");
    assert!(!stderr.is_empty(), "stderr should report the map not found");
    assert!(
        stderr.contains("Plain with rocks"),
        "stderr should list available maps, got: {stderr}"
    );
}

#[test]
fn test_cli_ticks_not_integer_exits_2() {
    let root = examples_dir();
    let (code, _, _) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "abc",
    ]);
    assert_eq!(code, 2, "--ticks not an integer should exit with 2");
}
