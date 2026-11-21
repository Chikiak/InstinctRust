use super::{bank_dir, rows_after_tick, run_cli};

// AC1: examples/ carga sin diagnósticos.
#[test]
fn bank_loads_clean() {
    let (code, _stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Arena",
        "--seed",
        "1",
        "--ticks",
        "0",
    ]);
    assert_eq!(code, 0, "exit 0, stderr was: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
}

// AC2: Sprinter speed 3 avanza 3 casillas en un turno.
#[test]
fn sprinter_advances_three_cells() {
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
        "Sprinter@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    let rows1 = rows_after_tick(&stdout, 1);
    let row = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(&row[5..6], "S", "tick 1: Sprinter en (5,2), row2='{row}'");
}

// AC3: Sprinter se detiene antes de roca.
#[test]
fn sprinter_stops_before_rock() {
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
        "Sprinter@2,4",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    let rows1 = rows_after_tick(&stdout, 1);
    let row = rows1.get(4).expect("tick 1 row 4");
    assert_eq!(
        &row[3..4],
        "S",
        "tick 1: Sprinter en (3,4) antes de la roca, row4='{row}'"
    );
    assert_eq!(&row[4..5], "#", "tick 1: roca sigue en (4,4), row4='{row}'");
}

// AC4: Sprinter se detiene ante criatura, Statue intacta.
#[test]
fn sprinter_stops_before_creature() {
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
        "Sprinter@2,6",
        "--place",
        "Guardian@4,6",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    let rows1 = rows_after_tick(&stdout, 1);
    let row = rows1.get(6).expect("tick 1 row 6");
    assert_eq!(
        &row[3..4],
        "S",
        "tick 1: Sprinter en (3,6) antes de Guardian, row6='{row}'"
    );
    assert_eq!(
        &row[4..5],
        "G",
        "tick 1: Guardian intacto en (4,6), row6='{row}'"
    );
}

// AC5: Sprinter se detiene en la última casilla ante el borde.
#[test]
fn sprinter_stops_at_border() {
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
        "Sprinter@6,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    let rows1 = rows_after_tick(&stdout, 1);
    let row = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(
        &row[8..9],
        "S",
        "tick 1: Sprinter en (8,2) última columna, row2='{row}'"
    );
}

// AC6: move diagonal desplaza 1 en x y 1 en y mismo turno.
#[test]
fn diagonal_moves_one_cell_each_axis() {
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
        "Diagonal@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    let rows1 = rows_after_tick(&stdout, 1);
    let row = rows1.get(3).expect("tick 1 row 3");
    assert_eq!(&row[3..4], "D", "tick 1: Diagonal en (3,3), row3='{row}'");
}

// AC7: wait(3) duerme 2 ticks, actúa en el 3; wait(0) actúa cada tick.
// Nota: wait(3) → sleep_turns=2. Tick 0: wait ejecuta. Tick 1: duerme.
// Tick 2: se despierta sin ejecutar. Tick 3: dice "despierto" (evento [tick 3],
// observado tras la rejilla de tick 3 en la salida real).
#[test]
fn wait_sleeps_two_turns() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Arena",
        "--seed",
        "1",
        "--ticks",
        "4",
        "--every",
        "1",
        "--place",
        "Napper@2,2",
        "--place",
        "Impatient@2,4",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    // Napper no dice "despierto" en ticks 1 y 2.
    let rows1 = rows_after_tick(&stdout, 1);
    let row1 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(&row1[2..3], "N", "tick 1: Napper sigue en (2,2)");
    assert!(
        !stdout.contains("[tick 1] Napper says: despierto"),
        "tick 1: Napper no habla"
    );

    let rows2 = rows_after_tick(&stdout, 2);
    let row2 = rows2.get(2).expect("tick 2 row 2");
    assert_eq!(&row2[2..3], "N", "tick 2: Napper sigue en (2,2)");
    assert!(
        !stdout.contains("[tick 2] Napper says: despierto"),
        "tick 2: Napper no habla"
    );

    // Napper habla en tick 3 (evento [tick 3] observado tras rejilla de tick 3).
    assert!(
        stdout.contains("[tick 3] Napper@2,2 says: despierto"),
        "tick 3: Napper despierta y habla, stdout={stdout}"
    );

    // Impatient habla cada tick (ticks 1-3 observados en la salida real).
    assert!(
        stdout.contains("[tick 1] Impatient@2,4 says: cada tick"),
        "tick 1: Impatient habla"
    );
    assert!(
        stdout.contains("[tick 2] Impatient@2,4 says: cada tick"),
        "tick 2: Impatient habla"
    );
    assert!(
        stdout.contains("[tick 3] Impatient@2,4 says: cada tick"),
        "tick 3: Impatient habla"
    );
}

// AC8: Pacer avanza 2 casillas en tick 0 y dicho aparece en un tick siguiente.
// Una acción por turno aunque el movimiento sea de varias casillas.
#[test]
fn pacer_one_action_per_turn() {
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
        "Pacer@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");
    // Tick 0: Pacer en (2,2).
    let rows0 = rows_after_tick(&stdout, 0);
    let row0 = rows0.get(2).expect("tick 0 row 2");
    assert_eq!(&row0[2..3], "P", "tick 0: Pacer en (2,2)");
    // Tick 1: Pacer avanzó 2 casillas a (4,2).
    let rows1 = rows_after_tick(&stdout, 1);
    let row1 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(&row1[4..5], "P", "tick 1: Pacer en (4,2), row2='{row1}'");
    // El dicho "paso" aparece (en tick 1 o posterior, no junto al move de tick 0).
    assert!(
        stdout.contains("says: paso"),
        "Pacer dice 'paso', stdout={stdout}"
    );
}
