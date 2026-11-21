//! Pruebas de escenario F6-03: reproducción, aborto natural y ciclo de
//! turnos observables desde la terminal (banco `examples/`).
//!
//! Convencion de ejecucion: `run_cli(&["run","--root",&bank_dir(),"--map","Arena","--seed","1","--ticks","N","--every","1","--place",...])`.
use super::{bank_dir, rows_after_tick, run_cli};

// AC3: la cría juega por primera vez en el tick siguiente al de su nacimiento.
#[test]
fn offspring_first_turn_next_tick() {
    // Mother@2,2 con reproduce(0, 1, 5): cría en (2,3) al final del tick 0.
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
        "Mother@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");

    // Tick 0: solo la madre en (2,2) (la cría aún no ha nacido).
    let rows0 = rows_after_tick(&stdout, 0);
    let row2 = rows0.get(2).expect("tick 0 row 2");
    assert_eq!(&row2[2..3], "M", "tick 0: Mother en (2,2), row2='{row2}'");
    assert_eq!(
        &row2[3..4],
        ".",
        "tick 0: (2,3) vacía antes del nacimiento, row2='{row2}'"
    );

    // Tick 1: la madre sigue en (2,2) y la cría (misma especie, 'M') está en (2,3).
    let rows1 = rows_after_tick(&stdout, 1);
    let row2 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(&row2[2..3], "M", "tick 1: Mother en (2,2), row2='{row2}'");
    let row3 = rows1.get(3).expect("tick 1 row 3");
    assert_eq!(&row3[2..3], "M", "tick 1: cría en (2,3), row3='{row3}'");
}

// AC4: la cría no hereda las variables de la madre.
#[test]
fn offspring_inherits_no_variables() {
    // Heir@2,2: legacy = 42; reproduce(0, 1, 5); say(legacy); goto start.
    // La cría nace sin variables, pero su propio programa asigna legacy al
    // iniciar, así que no debe haber error de ejecución.
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
        "Heir@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");
    assert!(
        !stdout.contains("RuntimeError"),
        "la cría no debe fallar con error de ejecución, stdout={stdout}"
    );
    assert!(
        !stdout.contains("dies of error"),
        "la cría no debe morir por error, stdout={stdout}"
    );

    // Tick 1: la madre ('H') sigue en (2,2) y la cría ('H') juega en (2,3).
    let rows1 = rows_after_tick(&stdout, 1);
    let row2 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(
        &row2[2..3],
        "H",
        "tick 1: Heir madre en (2,2), row2='{row2}'"
    );
    let row3 = rows1.get(3).expect("tick 1 row 3");
    assert_eq!(&row3[2..3], "H", "tick 1: cría en (2,3), row3='{row3}'");
}

// AC5: reproduce aborta con la casilla destino ocupada por criatura.
#[test]
fn cli_abort_cell_occupied_by_creature() {
    // Mother@2,2 + Victim@2,3: destino (2,3) ocupado, no nace cría.
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Arena",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Mother@2,2",
        "--place",
        "Victim@2,3",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");

    // Tick 1: la Victim sigue en (2,3) y no apareció cría.
    let rows1 = rows_after_tick(&stdout, 1);
    let row3 = rows1.get(3).expect("tick 1 row 3");
    assert_eq!(
        &row3[2..3],
        "V",
        "tick 1: (2,3) sigue ocupada por la Victim, sin cría, row3='{row3}'"
    );
    assert!(
        !stdout.contains("dies"),
        "la Victim no debe morir, stdout={stdout}"
    );
}
