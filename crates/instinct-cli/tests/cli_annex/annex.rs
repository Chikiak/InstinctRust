//! End-to-end terminal tests for the four annex examples (F3-04).
//! Runs the real `instinct-cli` binary with the four species placed on the
//! same map and checks exit code, empty stderr and determinism.

use super::{examples_dir, rows_after_tick, run_cli};

// ---------------------------------------------------------------------------
// AC6: the four annex species run together on the same map
// ---------------------------------------------------------------------------

#[test]
fn test_cli_runs_all_annex_examples() {
    let root = examples_dir();
    // plain.map is 7 rows x 20 columns; all chosen cells are free ('.') and
    // distinct.
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--seed",
        "7",
        "--ticks",
        "30",
        "--place",
        "Dwarf@1,0",
        "--place",
        "Ent@5,1",
        "--place",
        "Hobbit@10,0",
        "--place",
        "Uruk@12,3",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr should be empty, got: {stderr}");
    // The map header and all four species should appear at tick 0.
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout should contain tick 0 header, got:\n{stdout}"
    );
    let rows0 = rows_after_tick(&stdout, 0);
    assert!(
        rows0.iter().any(|r| r.contains('D')),
        "tick 0: Dwarf present, rows={rows0:?}"
    );
    assert!(
        rows0.iter().any(|r| r.contains('E')),
        "tick 0: Ent present, rows={rows0:?}"
    );
    assert!(
        rows0.iter().any(|r| r.contains('H')),
        "tick 0: Hobbit present, rows={rows0:?}"
    );
    assert!(
        rows0.iter().any(|r| r.contains('U')),
        "tick 0: Uruk present, rows={rows0:?}"
    );
}

// ---------------------------------------------------------------------------
// AC7: determinism — same seed, byte-identical output
// ---------------------------------------------------------------------------

#[test]
fn test_cli_annex_is_deterministic() {
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
            "30",
            "--place",
            "Dwarf@1,0",
            "--place",
            "Ent@5,1",
            "--place",
            "Hobbit@10,0",
            "--place",
            "Uruk@12,3",
        ]);
        assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
        assert!(stderr.is_empty(), "stderr should be empty, got: {stderr}");
        stdout
    };
    let out1 = run(7);
    let out2 = run(7);
    assert_eq!(
        out1, out2,
        "two runs with the same seed must be byte-identical"
    );
}
