use std::process::{Command, Stdio};

/// Directorio raíz de recursos de ejemplo, relativo al manifiesto de este crate.
fn ejemplos_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
}

/// Localiza el binario de la prueba de integración.
fn binario_instinct_cli() -> std::path::PathBuf {
    std::env::var("BIN_INSTINCT_CLI")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(env!("CARGO_BIN_EXE_instinct-cli")).to_path_buf())
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
fn test_cli_prints_map() {
    let root = ejemplos_dir();
    let (codigo, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
    ]);
    assert_eq!(codigo, 0, "salida debe ser 0, stderr: {stderr}");
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout debe contener '== tick 0 ==', obtuvo:\n{stdout}"
    );
    // Las siete filas del mapa, idénticas al archivo.
    let esperado = [
        "....................",
        "...##...............",
        "...##.....#.........",
        "..........#....###..",
        "...............#....",
        ".#..................",
        ".#........##........",
    ];
    for fila in esperado {
        assert!(
            stdout.contains(fila),
            "stdout debe contener la fila '{fila}', obtuvo:\n{stdout}"
        );
    }
    assert!(
        stderr.is_empty(),
        "stderr debe estar vacío para un mapa válido, obtuvo: {stderr}"
    );
}

#[test]
fn test_cli_unknown_map_exits_1() {
    let root = ejemplos_dir();
    let (codigo, _, stderr) =
        run_cli(&["run", "--root", &root, "--map", "No existe", "--ticks", "0"]);
    assert_eq!(codigo, 1, "mapa inexistente debe salir con 1");
    assert!(
        !stderr.is_empty(),
        "stderr debe informar el mapa no encontrado"
    );
    assert!(
        stderr.contains("Plain with rocks"),
        "stderr debe listar los mapas disponibles, obtuvo: {stderr}"
    );
}

#[test]
fn test_cli_ticks_no_entero_salida_2() {
    let root = ejemplos_dir();
    let (codigo, _, _) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "abc",
    ]);
    assert_eq!(codigo, 2, "--ticks no entero debe salir con 2");
}
