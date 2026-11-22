//! Pruebas de escenario F6-03: reproducción, aborto natural y ciclo de
//! turnos observables desde la terminal (banco `examples/`).
//!
//! Convencion de ejecucion: `run_cli(&["run","--root",&bank_dir(),"--map","Arena","--seed","1","--ticks","N","--every","1","--place",...])`.
use super::{bank_dir, rows_after_tick, run_cli};

// AC3: la cría juega por primera vez en el tick siguiente al de su nacimiento.
// FirstTurnMother dice "ready" al inicio de su turno; la cría nace al final
// del tick 0 y su primer turno (con su primer say) ocurre en el tick 1.
#[test]
fn offspring_first_turn_next_tick() {
    // FirstTurnMother@2,2 con say("ready"); reproduce(0, 1, 5); goto start.
    // La cría nace en (2,3) al final del tick 0 y juega su primer turno en
    // el tick 1.
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
        "FirstTurnMother@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");

    // Tick 0: solo la madre en (2,2); (2,3) vacía; la cría aún no ha jugado.
    let rows0 = rows_after_tick(&stdout, 0);
    let row2 = rows0.get(2).expect("tick 0 row 2");
    assert_eq!(&row2[2..3], "F", "tick 0: madre en (2,2), row2='{row2}'");
    // (2,3) vacía antes del nacimiento: leer fila y=3, columna x=2.
    let row3 = rows0.get(3).expect("tick 0 row 3");
    assert_eq!(
        &row3[2..3],
        ".",
        "tick 0: (2,3) vacía antes del nacimiento, row3='{row3}'"
    );
    // No hay evento de la cría en [tick 0] (aún no ha nacido/jugado).
    assert!(
        !stdout.contains("[tick 0] FirstTurnMother@2,3"),
        "la cría no debe tener eventos en tick 0, stdout={stdout}"
    );

    // Tick 1: la madre sigue en (2,2) y la cría ('F') apareció en (2,3).
    let rows1 = rows_after_tick(&stdout, 1);
    let row2 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(&row2[2..3], "F", "tick 1: madre en (2,2), row2='{row2}'");
    let row3 = rows1.get(3).expect("tick 1 row 3");
    assert_eq!(&row3[2..3], "F", "tick 1: cría en (2,3), row3='{row3}'");

    // La cría ejecutó su primera acción en tick 1 (y nunca en tick 0).
    assert!(
        stdout.contains("[tick 1] FirstTurnMother@2,3 says: ready"),
        "la cría debe decir 'ready' en su primer turno (tick 1), stdout={stdout}"
    );
    assert!(
        !stdout.contains("[tick 0] FirstTurnMother@2,3 says: ready"),
        "la cría no debe haber dicho 'ready' en tick 0, stdout={stdout}"
    );
}

// AC4: la cría no hereda las variables de la madre. La cría intenta leer
// `legacy` en su primer turno (tick 1) sin haberlo asignado, lo que genera
// un error de ejecución y la muerte de la cría.
#[test]
fn offspring_inherits_no_variables() {
    // Heir@2,2: solo la madre asigna legacy = 42 y se reproduce; la cría
    // lee `legacy` sin asignarlo → RuntimeError + Died en [tick 1].
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Arena",
        "--seed",
        "1",
        "--ticks",
        "3",
        "--every",
        "1",
        "--place",
        "Heir@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");

    // [tick 1]: la cría en (2,3) falla al leer variable no asignada.
    assert!(
        stdout.contains("[tick 1] Heir@2,3 error at line 8: unknown variable 'legacy'"),
        "falta RuntimeError de la cría al leer legacy, stdout={stdout}"
    );
    assert!(
        stdout.contains("[tick 1] Heir@2,3 dies of error"),
        "falta muerte por error de la cría, stdout={stdout}"
    );

    // Tick 1: la madre ('H') sigue en (2,2) y la cría ('H') nació en (2,3).
    let rows1 = rows_after_tick(&stdout, 1);
    let row2 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(
        &row2[2..3],
        "H",
        "tick 1: Heir madre en (2,2), row2='{row2}'"
    );
    let row3 = rows1.get(3).expect("tick 1 row 3");
    assert_eq!(&row3[2..3], "H", "tick 1: cría en (2,3), row3='{row3}'");

    // Tick 2: la cría murió, (2,3) vuelve a '.' y la madre sigue en (2,2).
    let rows2 = rows_after_tick(&stdout, 2);
    let row2 = rows2.get(2).expect("tick 2 row 2");
    assert_eq!(
        &row2[2..3],
        "H",
        "tick 2: madre sigue en (2,2), row2='{row2}'"
    );
    let row3 = rows2.get(3).expect("tick 2 row 3");
    assert_eq!(
        &row3[2..3],
        ".",
        "tick 2: (2,3) vacía tras muerte de la cría, row3='{row3}'"
    );
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
