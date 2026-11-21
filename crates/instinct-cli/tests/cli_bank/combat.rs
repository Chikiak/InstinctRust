// Pruebas de escenario F6-02: golpes, consumo y reservas observables
// desde la terminal (banco `examples/`).
//
// Convencion de ejecucion: `run_cli(&["run","--root",&bank_dir(),"--map","Arena","--seed","1","--ticks","N","--every","1","--place",...])`.
//
// Nota sobre las coordenadas: el mapa `Arena` es 9x9 (x en 0..9, y en 0..9,
// indices 0..=8 en ambos ejes), aunque el enunciado F6-02 lo describa como
// 10x10. Por eso las pruebas de "fuera del mapa" colocan a la criatura en la
// ultima columna valida (8) y hacen que su acción apunte a una celda fuera del
// mapa (x = 9), en lugar de colocar a la criatura fuera del mapa (lo cual
// fallaria el propio --place).

use super::{bank_dir, rows_after_tick, run_cli};

// AC1: Brawler (hp20) mata a Victim (hp5) en (3,2) y libera la celda.
#[test]
fn attack_kills_and_frees_cell() {
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
        "Brawler@2,2",
        "--place",
        "Victim@3,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");

    // Tick 0: ambas criaturas presentes en la rejilla inicial (row 2 = y=2).
    let rows0 = rows_after_tick(&stdout, 0);
    let row2 = rows0.get(2).expect("tick 0 row 2");
    assert_eq!(&row2[2..3], "B", "tick 0: Brawler en (2,2), row2='{row2}'");
    assert_eq!(&row2[3..4], "V", "tick 0: Victim en (3,2), row2='{row2}'");

    // Eventos de tick 0: daño y muerte de Victim por salud.
    assert!(
        stdout.contains("[tick 0] Brawler@2,2 damages 3,2 by 10"),
        "falta evento de daño en tick 0, stdout={stdout}"
    );
    assert!(
        stdout.contains("[tick 0] Victim@3,2 dies of health"),
        "falta evento de muerte en tick 0, stdout={stdout}"
    );

    // Tick 1: Brawler sigue en (2,2) y la celda (3,2) quedo libre (row 2 = y=2).
    let rows1 = rows_after_tick(&stdout, 1);
    let row2 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(&row2[2..3], "B", "tick 1: Brawler en (2,2), row2='{row2}'");
    assert_eq!(
        &row2[3..4],
        ".",
        "tick 1: celda (3,2) liberada, row2='{row2}'"
    );
}

// AC3: ataque a un objeto emite evento de daño.
#[test]
fn attack_object_emits_damaged() {
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
        "Miner@3,4",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");
    assert!(
        stdout.contains("[tick 0] Miner@3,4 damages 4,4 by 10"),
        "falta evento de daño a la roca, stdout={stdout}"
    );
}

// AC6: ataque fuera del mapa es silencioso (no emite "damages").
#[test]
fn attack_out_of_map_is_silent() {
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
        "Brawler@8,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacio, got: {stderr}");
    assert!(
        !stdout.contains("damages"),
        "ataque fuera del mapa no debe emitir 'damages', stdout={stdout}"
    );
}
