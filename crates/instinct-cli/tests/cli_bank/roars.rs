//! Scenarios observables desde la terminal: `roar` y `say` (AC1, AC2, AC5).

use super::{bank_dir, rows_after_tick, run_cli};

// AC1: un rugido emitido en el tick 0 aparece como evento de rugido con su texto.
#[test]
fn roar_emits_event() {
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
        "Shouter@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    assert!(
        stdout.contains("[tick 0] Shouter@2,2 roars: ¡auau!"),
        "stdout debe tener el evento de rugido en el tick 0, got:\n{stdout}"
    );
}

// AC2: al tick siguiente la oyente dice el texto del rugido y un conteo de uno;
// antes de que nadie ruja, dice el texto vacío y cero.
#[test]
fn listener_reads_last_roar_next_tick() {
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
        "Shouter@2,2",
        "--place",
        "Listener@4,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    // Tick 0: el rugido del tick 0 aún no está en roars_previous.
    assert!(
        stdout.contains("[tick 0] Listener@4,2 says: 0"),
        "tick 0: la oyente dice cero rugidos, got:\n{stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|line| line == "[tick 0] Listener@4,2 says: "),
        "tick 0: la oyente dice texto vacio (linea exacta), got:\n{stdout}"
    );
    // Tick 1: el rugido del tick 0 ya esta en roars_previous (vision 5, dist 2).
    assert!(
        stdout.contains("[tick 1] Listener@4,2 says: ¡auau!"),
        "tick 1: la oyente dice el texto del rugido, got:\n{stdout}"
    );
    assert!(
        stdout.contains("[tick 1] Listener@4,2 says: 1"),
        "tick 1: la oyente dice un conteo de uno, got:\n{stdout}"
    );
}

// AC5: `say` no consume turno: la cantante dice dos textos y se mueve en el
// mismo tick.
#[test]
fn say_does_not_consume_turn() {
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
        "Singer@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    assert!(
        stdout.contains("[tick 0] Singer@2,2 says: uno"),
        "tick 0: dice 'uno', got:\n{stdout}"
    );
    assert!(
        stdout.contains("[tick 0] Singer@2,2 says: dos"),
        "tick 0: dice 'dos', got:\n{stdout}"
    );
    // Al tick 1 la cantante ya avanzo a (3,2) en el mismo tick que dijo.
    let rows1 = rows_after_tick(&stdout, 1);
    let row = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(&row[3..4], "S", "tick 1: Singer en (3,2), row2='{row}'");
}
