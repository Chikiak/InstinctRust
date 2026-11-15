use super::{examples_dir, rows_after_tick, run_cli};

// AC1: attack kills or wounds adjacent creature (E2E from terminal).
#[test]
fn test_cli_e2e_attack_kills_adjacent_creature() {
    // Attacker (20 hp) at (0,0) attacks Victim (5 hp) at (1,0) with damage 10:
    // Victim dies in the same tick, its cell is left as terrain only.
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--place",
        "Attacker@0,0",
        "--place",
        "Victim@1,0",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    // Tick 0: both present.
    let rows0 = rows_after_tick(&stdout, 0);
    assert!(
        rows0.iter().any(|r| r.contains('A')),
        "tick 0: Attacker present, rows={rows0:?}"
    );
    assert!(
        rows0.iter().any(|r| r.contains('V')),
        "tick 0: Victim present, rows={rows0:?}"
    );
    assert!(
        stdout.contains("[tick 0] Attacker@0,0 daña 1,0 por 10"),
        "stderr with Damaged event, got: {stderr}"
    );
    assert!(
        stdout.contains("[tick 0] Victim@1,0 muere por vida"),
        "stderr with Died cause life, got: {stderr}"
    );
    // Tick 1: Victim gone, its cell (1,0) is terrain ('.').
    let rows1 = rows_after_tick(&stdout, 1);
    assert!(
        rows1.iter().any(|r| r.contains('A')),
        "tick 1: Attacker still present, rows={rows1:?}"
    );
    let tick1_row0 = rows1.first().expect("tick 1 row 0");
    assert_eq!(
        &tick1_row0[1..2],
        ".",
        "tick 1: cell (1,0) is left as terrain only, row0='{tick1_row0}'"
    );
}

// AC2: consume kills the victim and heals the attacker (no cap).
#[test]
fn test_cli_e2e_consume_kills_and_heals() {
    // Consumer (10 hp) at (0,0) consumes Food (5 hp) at (1,0): damage 10 kills
    // Food and the attacker heals +7 (no cap) -> 17 hp.
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--place",
        "Consumer@0,0",
        "--place",
        "Food@1,0",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    assert!(
        stdout.contains("[tick 0] Consumer@0,0 daña 1,0 por 10"),
        "stderr with Damaged event, got: {stderr}"
    );
    assert!(
        stdout.contains("[tick 0] Food@1,0 muere por vida"),
        "stderr with Died cause life, got: {stderr}"
    );
    // Tick 1: Consumer still present, Food gone and cell (1,0) free.
    let rows1 = rows_after_tick(&stdout, 1);
    assert!(
        rows1.iter().any(|r| r.contains('C')),
        "tick 1: Consumer present, rows={rows1:?}"
    );
    let tick1_row0 = rows1.first().expect("tick 1 row 0");
    assert_eq!(
        &tick1_row0[1..2],
        ".",
        "tick 1: cell (1,0) is left as terrain only, row0='{tick1_row0}'"
    );
}

// AC3: reproduce spawns offspring with age 0 on a free cell.
#[test]
fn test_cli_e2e_reproduce_birth() {
    // Mother (20 hp) at (1,1) reproduces at (1,2): offspring born at age 0,
    // health 5.
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--seed",
        "1",
        "--ticks",
        "2",
        "--place",
        "Mother@1,1",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    // Tick 0: mother only.
    let rows0 = rows_after_tick(&stdout, 0);
    assert!(
        rows0.iter().any(|r| r.contains('M')),
        "tick 0: Mother present, rows={rows0:?}"
    );
    assert!(
        !stdout.contains("Died"),
        "tick 0: no deaths, stderr={stderr}"
    );
    // Tick 1: offspring present at (1,2) (row 2, column 1) and mother still
    // there.
    let rows1 = rows_after_tick(&stdout, 1);
    assert!(
        rows1.iter().any(|r| r.contains('M')),
        "tick 1: Mother still present, rows={rows1:?}"
    );
    let tick1_row2 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(
        &tick1_row2[1..2],
        "M",
        "tick 1: offspring at (1,2), row2='{tick1_row2}'"
    );
}

// AC4: roar emits Roared event (text on the following tick).
#[test]
fn test_cli_e2e_roar_emits_event() {
    // Roarer at (1,1) roars in tick 0: Roared event with text "¡auau!".
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--place",
        "Roarer@1,1",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    assert!(
        stdout.contains("[tick 0] Roarer@1,1 ruge: ¡auau!"),
        "stderr with Roared event, got: {stderr}"
    );
}
