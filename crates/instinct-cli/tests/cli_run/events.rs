use super::{examples_dir, run_cli};

// AC4: events on stdout via CliOutput, load diagnostics on stderr.
#[test]
fn test_cli_events_stdout_diagnostics_stderr() {
    let root = examples_dir();
    // DivZero has `n = 1 / 0` -> runtime error on the first turn.
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--seed",
        "7",
        "--ticks",
        "1",
        "--place",
        "DivZero@0,0",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    // The RuntimeError event must go to stdout (via CliOutput).
    assert!(
        stdout.contains("error at line") || stdout.contains("division by zero"),
        "stdout must contain the RuntimeError event, got:\n{stdout}"
    );
    // No load diagnostics must go to stdout.
    assert!(
        !stdout.contains("diagnostic") && !stdout.contains("error:"),
        "stdout must not contain load diagnostics, got:\n{stdout}"
    );
    // Load diagnostics (if any) go to stderr.
    // For a valid run there are no load diagnostics.
}

// AC5: determinism — two runs with the same seed, byte-identical output.
#[test]
fn test_cli_determinism_same_seed() {
    let root = examples_dir();
    let run = |seed: u64| {
        let (code, stdout, stderr) = run_cli(&[
            "run",
            "--root",
            &root,
            "--map",
            "Plain with rocks",
            "--seed",
            &seed.to_string(),
            "--ticks",
            "10",
            "--place",
            "Walker@0,0",
        ]);
        assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
        (stdout, stderr)
    };
    let (out1, err1) = run(1);
    let (out2, err2) = run(1);
    assert_eq!(
        out1, out2,
        "two runs with the same seed give identical stdout"
    );
    assert_eq!(
        err1, err2,
        "two runs with the same seed give identical stderr"
    );
}
