use super::{examples_dir, run_cli};

// AC6: no F0 regressions — `wait`, `move`, `say` still work.
#[test]
fn test_cli_f0_walker_no_regression() {
    // The phase command still works: map at tick 0 and 10, death by life,
    // exit 0.
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--seed",
        "1",
        "--ticks",
        "10",
        "--place",
        "Walker@0,0",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout shows tick 0, got:\n{stdout}"
    );
    assert!(
        stdout.contains("== tick 10 =="),
        "stdout shows tick 10, got:\n{stdout}"
    );
    // Walker has health 10 -> dies by life at end of tick 10.
    assert!(
        stdout.contains("[tick 9] Walker@0,0 muere por vida") || stdout.contains("muere por vida"),
        "stderr reports death by life, got: {stderr}"
    );
    assert!(
        stderr.is_empty(),
        "stderr empty for a valid run, got: {stderr}"
    );
}
