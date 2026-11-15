use super::run_cli;

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
        "stdout should contain the usage text, got: {stderr}"
    );
    assert!(
        stdout.contains("run") && stdout.contains("check"),
        "usage should mention run and check"
    );
}

#[test]
fn test_cli_unknown_subcommand_exits_2() {
    let (code, _, stderr) = run_cli(&["missing"]);
    assert_eq!(code, 2, "unimplemented subcommand should exit with 2");
    assert!(
        !stderr.is_empty(),
        "unknown subcommand should write a message to stderr"
    );
}
