//! Mapas invalidos: caracter desconocido y fila dispareja (AC9, AC10).

use super::run_cli;

// AC9: el mapa con caracter desconocido produce un diagnostico que nombra el
// caracter y la linea, y el mapa se puede simular.
#[test]
fn unknown_char_reports_and_loads() {
    let root = super::traps_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "UnknownChar",
        "--seed",
        "1",
        "--ticks",
        "2",
        "--every",
        "1",
        "--place",
        "Valid@2,2",
    ]);
    assert_eq!(code, 0, "UnknownChar debe salir 0, stderr: {stderr}");
    // El diagnostico nombra el caracter y la linea de su fila (sale al cargar
    // el mapa y otra al construir el mundo).
    assert!(
        stderr.contains("unknown character in the map: 'X'"),
        "stderr debe nombrar el caracter desconocido, got: {stderr}"
    );
    assert!(
        stderr.contains("UnknownChar:5: unknown character in the map: 'X'")
            || stderr.contains("unknown_char.map:5: unknown character in the map: 'X'"),
        "stderr debe tener archivo:linea: mensaje del caracter, got: {stderr}"
    );
    // El mapa se puede simular: la valida aparece en la rejilla.
    assert!(
        stdout.contains("..V......"),
        "tick 0: Valid en (2,2) sobre UnknownChar, got:\n{stdout}"
    );
}

// AC10: el mapa con fila dispareja produce un diagnostico con la linea de la
// fila que no cuadra.
#[test]
fn ragged_row_reports_line() {
    let root = super::traps_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Ragged",
        "--seed",
        "1",
        "--ticks",
        "2",
        "--every",
        "1",
        "--place",
        "Valid@2,2",
    ]);
    assert_eq!(code, 0, "Ragged debe salir 0, stderr: {stderr}");
    assert!(
        stderr.contains("ragged.map:4: the row has 10 characters, 9 were expected"),
        "stderr debe reportar la fila dispareja con su linea, got: {stderr}"
    );
    // El mapa se puede simular (la fila dispareja se salta al cargar).
    assert!(
        stdout.contains("..V......"),
        "tick 0: Valid en (2,2) sobre Ragged, got:\n{stdout}"
    );
}
