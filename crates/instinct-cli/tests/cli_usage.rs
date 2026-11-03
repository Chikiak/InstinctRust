use std::process::{Command, Stdio};

/// Localiza el binario de la prueba de integración mediante la variable de
/// entorno que Cargo proporciona: `BIN_INSTINCT_CLI` (establecida por
/// #[test] con `#[path]` o por el harness de cargo test).
fn binario_instinct_cli() -> std::path::PathBuf {
    std::env::var("BIN_INSTINCT_CLI")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            // Fallback: buscar en target/debug (cargo test lo pone ahí).
            std::path::Path::new(env!("CARGO_BIN_EXE_instinct-cli")).to_path_buf()
        })
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(binario_instinct_cli())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("no se pudo ejecutar el binario instinct-cli");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn test_cli_without_args_exits_2() {
    let (codigo, stdout, stderr) = run_cli(&[]);
    assert_eq!(codigo, 2, "sin argumentos debe salir con 2");
    assert!(
        stdout.is_empty(),
        "sin argumentos no debe escribir en stdout"
    );
    assert!(
        !stderr.is_empty(),
        "sin argumentos debe escribir el uso en stderr"
    );
    assert!(
        stderr.contains("Uso:"),
        "stderr debe contener el texto de uso, obtuvo: {stderr}"
    );
}

#[test]
fn test_cli_help_exits_0() {
    let (codigo, stdout, stderr) = run_cli(&["--help"]);
    assert_eq!(codigo, 0, "--help debe salir con 0");
    assert!(!stdout.is_empty(), "--help debe escribir el uso en stdout");
    assert!(stderr.is_empty(), "--help no debe escribir en stderr");
    assert!(
        stdout.contains("Uso:"),
        "stdout debe contener el texto de uso, obtuvo: {stdout}"
    );
    assert!(
        stdout.contains("run") && stdout.contains("check"),
        "el uso debe mencionar run y check"
    );
}

#[test]
fn test_cli_unknown_subcommand_exits_2() {
    let (codigo, _, stderr) = run_cli(&["run", "--root", ".", "--map", "x"]);
    assert_eq!(codigo, 2, "subcomando no implementado debe salir con 2");
    assert!(
        !stderr.is_empty(),
        "subcomando desconocido debe escribir mensaje en stderr"
    );
}
