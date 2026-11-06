use instinct_core::actions::{Action, Consume};
use instinct_core::resources::{Map, Terrain};
use instinct_core::value::Value;
use instinct_core::world::World;
use std::process::{Command, Stdio};

/// Example resources root directory, relative to this crate's manifest.
fn examples_dir() -> String {
    format!("{}/../../examples", env!("CARGO_MANIFEST_DIR"))
}

/// Locates the integration test binary.
fn instinct_cli_binary() -> std::path::PathBuf {
    std::env::var("BIN_INSTINCT_CLI")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::Path::new(env!("CARGO_BIN_EXE_instinct-cli")).to_path_buf())
}

fn run_cli(args: &[&str]) -> (i32, String, String) {
    let output = Command::new(instinct_cli_binary())
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("could not run the instinct-cli binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

#[test]
fn test_cli_prints_map() {
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
    ]);
    assert_eq!(code, 0, "exit code should be 0, stderr: {stderr}");
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout should contain '== tick 0 ==', got:\n{stdout}"
    );
    // The seven rows of the map, identical to the file.
    let expected = [
        "....................",
        "...##...............",
        "...##.....#.........",
        "..........#....###..",
        "...............#....",
        ".#..................",
        ".#........##........",
    ];
    for row in expected {
        assert!(
            stdout.contains(row),
            "stdout should contain the row '{row}', got:\n{stdout}"
        );
    }
    assert!(
        stderr.is_empty(),
        "stderr should be empty for a valid map, got: {stderr}"
    );
}

#[test]
fn test_cli_unknown_map_exits_1() {
    let root = examples_dir();
    let (code, _, stderr) =
        run_cli(&["run", "--root", &root, "--map", "No existe", "--ticks", "0"]);
    assert_eq!(code, 1, "nonexistent map should exit with 1");
    assert!(!stderr.is_empty(), "stderr should report the map not found");
    assert!(
        stderr.contains("Plain with rocks"),
        "stderr should list available maps, got: {stderr}"
    );
}

#[test]
fn test_cli_ticks_not_integer_exits_2() {
    let root = examples_dir();
    let (code, _, _) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "abc",
    ]);
    assert_eq!(code, 2, "--ticks not an integer should exit with 2");
}

// ---------------------------------------------------------------------------
// Helpers para los escenarios E2E de F1.
// ---------------------------------------------------------------------------

/// Fichas de la rejilla ASCII que aparecen tras la cabecera `== tick N ==`,
/// hasta la siguiente cabecera de tick (o el final de la salida).
fn rows_after_tick(stdout: &str, tick: u64) -> Vec<String> {
    let marker = format!("== tick {tick} ==");
    let mut rows: Vec<String> = Vec::new();
    let mut found = false;
    for line in stdout.lines() {
        if line == marker {
            found = true;
            continue;
        }
        if found {
            if line.starts_with("== tick ") {
                break;
            }
            rows.push(line.to_string());
        }
    }
    rows
}

/// Mundo mínimo de hierba (5x5) con el que se construye el World a mano.
/// El mundo se queda con los catálogos (dueño único).
fn grass_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
        ],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 1,
        reserve: 100,
    }];
    let objects = vec![];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

/// Coloca una criatura con la salud indicada (Species::new tiene health 0).
fn place_with_health(
    world: &mut World,
    x: usize,
    y: usize,
    health: i64,
) -> instinct_core::world::CreatureId {
    let id = world
        .place_creature(instinct_core::species::Species::new("Walker"), x, y)
        .expect("place creature");
    for c in world.creatures.iter_mut() {
        if c.id == id {
            c.health = health;
        }
    }
    id
}

/// Devuelve la salud de una criatura viva; falla si no existe.
fn creature_health(world: &World, id: instinct_core::world::CreatureId) -> i64 {
    world
        .creatures
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.health)
        .expect("creature exists")
}

// ---------------------------------------------------------------------------
// AC1: attack mata o hiere a criatura adyacente (E2E desde terminal).
// ---------------------------------------------------------------------------

#[test]
fn test_cli_e2e_attack_kills_adjacent_creature() {
    // Attacker (20 hp) en (0,0) ataca a Victim (5 hp) en (1,0) con daño 10:
    // Victim muere en el mismo tick, su casilla queda solo terreno.
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
    // Tick 0: ambos presentes.
    let rows0 = rows_after_tick(&stdout, 0);
    assert!(
        rows0.iter().any(|r| r.contains('A')),
        "tick 0: Attacker presente, rows={rows0:?}"
    );
    assert!(
        rows0.iter().any(|r| r.contains('V')),
        "tick 0: Victim presente, rows={rows0:?}"
    );
    assert!(
        stdout.contains("[tick 0] Attacker@0,0 daña 1,0 por 10"),
        "stderr con evento Damaged, got: {stderr}"
    );
    assert!(
        stdout.contains("[tick 0] Victim@1,0 muere por vida"),
        "stderr con evento Died causa vida, got: {stderr}"
    );
    // Tick 1: Victim desaparecida, su casilla (1,0) es terreno ('.').
    let rows1 = rows_after_tick(&stdout, 1);
    assert!(
        rows1.iter().any(|r| r.contains('A')),
        "tick 1: Attacker sigue presente, rows={rows1:?}"
    );
    let tick1_row0 = rows1.first().expect("tick 1 row 0");
    assert_eq!(
        &tick1_row0[1..2],
        ".",
        "tick 1: casilla (1,0) queda solo terreno, row0='{tick1_row0}'"
    );
}

// ---------------------------------------------------------------------------
// AC2: consume mata a la víctima y cura al atacante (sin tope).
// ---------------------------------------------------------------------------

#[test]
fn test_cli_e2e_consume_kills_and_heals() {
    // Consumer (10 hp) en (0,0) consume a Food (5 hp) en (1,0): daño 10 mata
    // a Food y el atacante se cura +7 (sin tope) -> 17 hp.
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
        "stderr con evento Damaged, got: {stderr}"
    );
    assert!(
        stdout.contains("[tick 0] Food@1,0 muere por vida"),
        "stderr con evento Died causa vida, got: {stderr}"
    );
    // Tick 1: Consumer sigue presente, Food desaparecida y casilla (1,0) libre.
    let rows1 = rows_after_tick(&stdout, 1);
    assert!(
        rows1.iter().any(|r| r.contains('C')),
        "tick 1: Consumer presente, rows={rows1:?}"
    );
    let tick1_row0 = rows1.first().expect("tick 1 row 0");
    assert_eq!(
        &tick1_row0[1..2],
        ".",
        "tick 1: casilla (1,0) queda solo terreno, row0='{tick1_row0}'"
    );
}

// ---------------------------------------------------------------------------
// AC3: reproduce cría con edad 0 en casilla libre.
// ---------------------------------------------------------------------------

#[test]
fn test_cli_e2e_reproduce_birth() {
    // Mother (20 hp) en (1,1) se reproduce en (1,2): cría nace edad 0, vida 5.
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
    // Tick 0: solo la madre.
    let rows0 = rows_after_tick(&stdout, 0);
    assert!(
        rows0.iter().any(|r| r.contains('M')),
        "tick 0: Mother presente, rows={rows0:?}"
    );
    assert!(
        !stdout.contains("Died"),
        "tick 0: sin muertes, stderr={stderr}"
    );
    // Tick 1: cría presente en (1,2) (fila 2, columna 1) y madre sigue.
    let rows1 = rows_after_tick(&stdout, 1);
    assert!(
        rows1.iter().any(|r| r.contains('M')),
        "tick 1: Mother sigue presente, rows={rows1:?}"
    );
    let tick1_row2 = rows1.get(2).expect("tick 1 row 2");
    assert_eq!(
        &tick1_row2[1..2],
        "M",
        "tick 1: cría en (1,2), row2='{tick1_row2}'"
    );
}

// ---------------------------------------------------------------------------
// AC4: roar emite evento Roared (texto en el tick siguiente).
// ---------------------------------------------------------------------------

#[test]
fn test_cli_e2e_roar_emits_event() {
    // Roarer en (1,1) ruge en tick 0: evento Roared con texto "¡auau!".
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
        "stderr con evento Roared, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC5: determinismo — dos corridas misma semilla, salida byte a byte idéntica.
// ---------------------------------------------------------------------------

#[test]
fn test_cli_determinism_same_seed() {
    let root = examples_dir();
    let run = |seed: u64| {
        let (code, stdout, stderr) = run_cli(&[
            "run",
            "--root",
            &root,
            "--map",
            "Plain with rocks",
            "--seed",
            &seed.to_string(),
            "--ticks",
            "10",
            "--place",
            "Walker@0,0",
        ]);
        assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
        (stdout, stderr)
    };
    let (out1, err1) = run(1);
    let (out2, err2) = run(1);
    assert_eq!(out1, out2, "dos corridas misma semilla dan stdout idéntico");
    assert_eq!(err1, err2, "dos corridas misma semilla dan stderr idéntico");
}

// ---------------------------------------------------------------------------
// AC6: sin regresiones de F0 — `wait`, `move`, `say` siguen funcionando.
// ---------------------------------------------------------------------------

#[test]
fn test_cli_f0_walker_no_regression() {
    // El comando de la fase sigue funcionando: mapa en tick 0 y 10, muerte
    // por vida, exit 0.
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
        "10",
        "--place",
        "Walker@0,0",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout muestra tick 0, got:\n{stdout}"
    );
    assert!(
        stdout.contains("== tick 10 =="),
        "stdout muestra tick 10, got:\n{stdout}"
    );
    // Walker tiene salud 10 -> muere por vida al final del tick 10.
    assert!(
        stdout.contains("[tick 9] Walker@0,0 muere por vida") || stdout.contains("muere por vida"),
        "stderr reporta muerte por vida, got: {stderr}"
    );
    assert!(
        stderr.is_empty(),
        "stderr vacío para corrida válida, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Batería C4: casos de validación de argumentos de `run`.
// ---------------------------------------------------------------------------

#[test]
fn test_cli_missing_map_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&["run", "--root", &root, "--ticks", "0"]);
    assert_eq!(code, 2, "run sin --map debe salir con 2, stderr: {stderr}");
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar la falta de --map, got: {stderr}"
    );
    assert!(
        stderr.contains("--map"),
        "stderr debe nombrar --map, got: {stderr}"
    );
}

#[test]
fn test_cli_negative_ticks_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "-3",
    ]);
    assert_eq!(
        code, 2,
        "--ticks negativo debe salir con 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );
    assert!(
        stderr.contains("--ticks") || stderr.contains("ticks"),
        "stderr debe nombrar --ticks, got: {stderr}"
    );
}

#[test]
fn test_cli_missing_root_errors() {
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        "no/existe/este/directorio",
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
    ]);
    assert_eq!(
        code, 2,
        "--root inexistente debe salir con 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );
    assert!(
        stderr.contains("no/existe/este/directorio"),
        "stderr debe nombrar el directorio, got: {stderr}"
    );
}

#[test]
fn test_cli_every_zero_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "2",
        "--every",
        "0",
    ]);
    assert_eq!(code, 2, "--every 0 debe salir con 2, stderr: {stderr}");
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );
}

#[test]
fn test_cli_malformed_place_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--place",
        "formato-mal",
    ]);
    assert_eq!(
        code, 1,
        "--place mal formado debe salir con 1, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );
}

#[test]
fn test_cli_unknown_species_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--place",
        "NoExiste@0,0",
    ]);
    assert_eq!(
        code, 1,
        "especie desconocida debe salir con 1, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );
    assert!(
        stderr.contains("NoExiste"),
        "stderr debe nombrar la especie, got: {stderr}"
    );
}

#[test]
fn test_cli_place_out_of_bounds_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--place",
        "Walker@9999,9999",
    ]);
    assert_eq!(
        code, 1,
        "colocación fuera de límites debe salir con 1, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );
}

#[test]
fn test_cli_streams_separated() {
    let root = examples_dir();
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
    ]);
    assert_eq!(code, 0, "exit code 0, stderr: {stderr}");
    // La simulación (mapa) va por stdout.
    assert!(
        stdout.contains("== tick 0 =="),
        "stdout debe contener el mapa, got:\n{stdout}"
    );
    // Los diagnósticos de carga van por stderr; para un mapa válido está vacío.
    assert!(
        stderr.is_empty(),
        "stderr debe estar vacío para corrida válida, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// AC1: flag desconocido (delegado a cli_usage.rs, pero se verifica aquí que no
// se ignore silenciosamente).
// ---------------------------------------------------------------------------

#[test]
fn test_cli_unknown_flag_errors() {
    let root = examples_dir();
    let (code, _, stderr) = run_cli(&[
        "run",
        "--root",
        &root,
        "--map",
        "Plain with rocks",
        "--ticks",
        "0",
        "--bandera-inexistente",
    ]);
    assert_eq!(
        code, 2,
        "flag desconocido debe salir con 2, stderr: {stderr}"
    );
    assert!(
        !stderr.is_empty(),
        "stderr debe reportar el error, got: {stderr}"
    );
}

// ---------------------------------------------------------------------------
// Core: consume cura al atacante sin tope (prueba de simulación directa).
// ---------------------------------------------------------------------------

#[test]
fn test_core_consume_heals_without_cap() {
    // AC2 de F1-02: heal se acumula sin tope (10 + 7 = 17).
    let mut world = grass_world();
    let attacker = place_with_health(&mut world, 0, 0, 10);
    let victim = place_with_health(&mut world, 1, 0, 10);

    let consume = Consume;
    let r = consume
        .apply(
            &mut world,
            attacker,
            &[Value::Int(1), Value::Int(0), Value::Int(3), Value::Int(7)],
        )
        .expect("consume applies");
    assert_eq!(r.sleep_turns, 0);
    assert_eq!(creature_health(&world, victim), 7, "víctima herida");
    assert_eq!(
        creature_health(&world, attacker),
        17,
        "atacante curado +7 sin tope"
    );
}
