// Pruebas de escenario F6-05 (CLI): errores de ejecución observables desde la
// terminal sobre el banco `examples/`. Siempre salen con exit 0: la criatura
// que falla muere, la simulación continúa.

use super::{bank_dir, rows_after_tick, run_cli};

fn run_place(species: &str, x: usize, y: usize, ticks: u64) -> (i32, String, String) {
    run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Arena",
        "--seed",
        "1",
        "--ticks",
        &ticks.to_string(),
        "--every",
        "1",
        "--place",
        &format!("{species}@{x},{y}"),
    ])
}

// AC4: comparar textos con operador de orden muere con error y línea.
#[test]
fn text_order_comparison_dies() {
    let (code, stdout, stderr) = run_place("TextOrder", 2, 2, 2);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");
    assert!(
        stdout.contains("[tick 0] TextOrder@2,2 error at line 7: cannot compare texts with an ordering operator"),
        "falta error de orden sobre textos, stdout={stdout}"
    );
    assert!(
        stdout.contains("[tick 0] TextOrder@2,2 dies of error"),
        "falta muerte por error, stdout={stdout}"
    );
}

// AC5: mezclar entero y texto muere con error y línea, celda queda libre.
#[test]
fn mixed_types_die_with_line() {
    let (code, stdout, stderr) = run_place("Mixer", 2, 2, 2);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");
    assert!(
        stdout.contains("[tick 0] Mixer@2,2 error at line 7: cannot add an integer and a text"),
        "falta error de mezcla de tipos, stdout={stdout}"
    );
    assert!(
        stdout.contains("[tick 0] Mixer@2,2 dies of error"),
        "falta muerte por error, stdout={stdout}"
    );
    // Tick 1: la celda (2,2) quedo libre.
    let rows1 = rows_after_tick(&stdout, 1);
    let row2 = rows1.get(2).expect("tick 1 row 2 (y=2)");
    assert_eq!(
        &row2[2..3],
        ".",
        "tick 1: celda (2,2) liberada, row2='{row2}'"
    );
}

// AC6: división por cero y variable sin asignar matan con mensaje y línea.
#[test]
fn div_zero_and_unassigned_die() {
    for (species, expected) in [
        (
            "DividerZero",
            "[tick 0] DividerZero@2,2 error at line 7: division by zero",
        ),
        (
            "Forgetful",
            "[tick 0] Forgetful@2,2 error at line 7: unknown variable 'unassigned_var'",
        ),
    ] {
        let (code, stdout, stderr) = run_place(species, 2, 2, 2);
        assert_eq!(code, 0, "{species}: exit 0, stderr: {stderr}");
        assert!(stderr.is_empty(), "{species}: stderr vacio, got: {stderr}");
        assert!(
            stdout.contains(expected),
            "{species}: falta error esperado '{expected}', stdout={stdout}"
        );
        assert!(
            stdout.contains(&format!("[tick 0] {species}@2,2 dies of error")),
            "{species}: falta muerte por error, stdout={stdout}"
        );
    }
}

// AC7: texto donde se espera número y dirección fuera de rango matan con mensaje y línea.
#[test]
fn text_argument_and_bad_direction_die() {
    for (species, expected) in [
        (
            "Wordy",
            "[tick 0] Wordy@2,2 error at line 7: argument 'dx' must be an integer, got text",
        ),
        (
            "OutOfRange",
            "[tick 0] OutOfRange@2,2 error at line 7: argument 'dx' must be -1, 0, or 1, got 2",
        ),
    ] {
        let (code, stdout, stderr) = run_place(species, 2, 2, 2);
        assert_eq!(code, 0, "{species}: exit 0, stderr: {stderr}");
        assert!(stderr.is_empty(), "{species}: stderr vacio, got: {stderr}");
        assert!(
            stdout.contains(expected),
            "{species}: falta error esperado '{expected}', stdout={stdout}"
        );
        assert!(
            stdout.contains(&format!("[tick 0] {species}@2,2 dies of error")),
            "{species}: falta muerte por error, stdout={stdout}"
        );
    }
}

// AC8: un error de ejecución mata solo a esa criatura; la otra sigue viva.
#[test]
fn runtime_error_kills_only_its_creature() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Arena",
        "--seed",
        "1",
        "--ticks",
        "2",
        "--every",
        "1",
        "--place",
        "Wordy@2,2",
        "--place",
        "Victim@6,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");
    assert!(
        stdout.contains(
            "[tick 0] Wordy@2,2 error at line 7: argument 'dx' must be an integer, got text"
        ),
        "falta error de Wordy, stdout={stdout}"
    );
    assert!(
        stdout.contains("[tick 0] Wordy@2,2 dies of error"),
        "falta muerte de Wordy, stdout={stdout}"
    );
    // La víctima sigue viva en la casilla (6,2) en el tick 1 (row y=2, col x=6).
    let rows1 = rows_after_tick(&stdout, 1);
    let row2 = rows1.get(2).expect("tick 1 row 2 (y=2)");
    assert_eq!(
        &row2[6..7],
        "V",
        "tick 1: víctima sigue en (6,2), row2='{row2}'"
    );
    // La casilla de Wordy (2,2) quedó libre.
    assert_eq!(
        &row2[2..3],
        ".",
        "tick 1: celda (2,2) liberada, row2='{row2}'"
    );
}
