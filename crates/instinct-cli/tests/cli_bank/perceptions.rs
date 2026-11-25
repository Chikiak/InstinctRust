// Pruebas de escenario F6-04 (CLI): líneas de registro emitidas por las
// criaturas del banco sobre percepciones propias, de mundo, de cercanía,
// `see` y `name`, sobre el mapa `mixed.map` (9x9).
//
// Cada test ejecuta el binario real de la CLI con `--seed 1` y comprueba que
// aparecen las líneas `[tick N] <Especie>@<x>,<y> says: <valor>` esperadas
// en el orden exacto de emisión del programa `.ins`.

use super::{bank_dir, run_cli};

/// Devuelve todas las líneas `[tick N] <Especie>@<x>,<y> says: <valor>` en
/// orden de emisión.
fn says_lines(stdout: &str, tick: u64, species: &str, x: i64, y: i64) -> Vec<String> {
    let prefix = format!("[tick {tick}] {species}@{x},{y} says: ");
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .map(|s| s.trim().to_string())
        .collect()
}

// ---------------------------------------------------------------------------
// AC1: percepciones propias coinciden con la colocación y la cabecera.
//
// scout.ins emite (tick 0, Scout@2,2):
//   0 health=20  1 age=0  2 x=2  3 y=2  4 species="Scout"  5 faction="explorers"
//   6 terrain_here="Grass"  7 reserve_here=100  8 width=9  9 height=9
//   10 total_creatures=0  11 total_allies=0  12 total_enemies=0
//
#[test]
fn cli_self_perceptions_say_placement() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Scout@2,2",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    let s = says_lines(&stdout, 0, "Scout", 2, 2);
    assert_eq!(s.first(), Some(&"20".to_string()), "health");
    assert_eq!(s.get(1), Some(&"0".to_string()), "age");
    assert_eq!(s.get(2), Some(&"2".to_string()), "x");
    assert_eq!(s.get(3), Some(&"2".to_string()), "y");
    assert_eq!(s.get(4), Some(&"Scout".to_string()), "species");
    assert_eq!(s.get(5), Some(&"explorers".to_string()), "facción");
    assert_eq!(
        s.get(6),
        Some(&"Grass".to_string()),
        "terreno bajo el explorador"
    );
    assert_eq!(s.get(7), Some(&"100".to_string()), "reserva de hierba");
    assert_eq!(s.get(8), Some(&"9".to_string()), "ancho del mapa");
    assert_eq!(s.get(9), Some(&"9".to_string()), "alto del mapa");
    assert_eq!(
        s.get(10),
        Some(&"0".to_string()),
        "total_creatures (solo él)"
    );
    assert_eq!(s.get(11), Some(&"0".to_string()), "total_allies");
    assert_eq!(s.get(12), Some(&"0".to_string()), "total_enemies");
}

// ---------------------------------------------------------------------------
// AC2: totales del mundo excluyen a la propia criatura.
//
// Scout@2,2 + Companion@5,5 + Stranger@7,7 (tick 0):
//   10 total_creatures=2  11 total_allies=1  12 total_enemies=1
//
#[test]
fn cli_world_perceptions_count_others() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Scout@2,2",
        "--place",
        "Companion@5,5",
        "--place",
        "Stranger@7,7",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    let s = says_lines(&stdout, 0, "Scout", 2, 2);
    assert_eq!(
        s.get(10),
        Some(&"2".to_string()),
        "total_creatures (2 otros)"
    );
    assert_eq!(
        s.get(11),
        Some(&"1".to_string()),
        "total_allies (Companion)"
    );
    assert_eq!(
        s.get(12),
        Some(&"1".to_string()),
        "total_enemies (Stranger)"
    );
    assert_eq!(s.get(8), Some(&"9".to_string()), "width");
    assert_eq!(s.get(9), Some(&"9".to_string()), "height");
}

// ---------------------------------------------------------------------------
// AC3: conteos de cercanía excluyen la casilla propia.
//
// myope.ins emite (tick 0, Myope@5,4 + Companion@5,5 + Stranger@6,4):
//   5 allies_near=1  6 enemies_near=1  7 objects_near=1
//
#[test]
fn cli_near_counts_exclude_own_cell() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Myope@5,4",
        "--place",
        "Companion@5,5",
        "--place",
        "Stranger@6,4",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    let s = says_lines(&stdout, 0, "Myope", 5, 4);
    assert_eq!(
        s.get(5),
        Some(&"1".to_string()),
        "allies_near (Companion en (5,5))"
    );
    assert_eq!(
        s.get(6),
        Some(&"1".to_string()),
        "enemies_near (Stranger en (6,4))"
    );
    assert_eq!(
        s.get(7),
        Some(&"1".to_string()),
        "objects_near (cristal en (4,5))"
    );
}

// ---------------------------------------------------------------------------
// AC4: distancia y dirección al más cercano.
//
// scout.ins emite:
//   Caso A (sin objetivo): 13 ally_dist=-1  14 ally_dx=0  15 ally_dy=0
//                          16 enemy_dist=-1  17 enemy_dx=0  18 enemy_dy=0
//   Caso B (con Stranger@7,4): 16 enemy_dist=3  17 enemy_dx=1  18 enemy_dy=0
//
#[test]
fn cli_nearest_directions_and_distance() {
    // A: sin objetivo visible → enemy_dist -1, direcciones 0.
    let (code_a, stdout_a, stderr_a) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Scout@4,4",
    ]);
    assert_eq!(code_a, 0, "exit 0, stderr: {stderr_a}");
    assert!(stderr_a.is_empty(), "stderr vacío, got: {stderr_a}");

    let sa = says_lines(&stdout_a, 0, "Scout", 4, 4);
    assert_eq!(
        sa.get(16),
        Some(&"-1".to_string()),
        "enemy_dist sin objetivo"
    );
    assert_eq!(sa.get(17), Some(&"0".to_string()), "enemy_dx sin objetivo");
    assert_eq!(sa.get(18), Some(&"0".to_string()), "enemy_dy sin objetivo");
    assert_eq!(
        sa.get(13),
        Some(&"-1".to_string()),
        "ally_dist sin objetivo"
    );
    assert_eq!(sa.get(14), Some(&"0".to_string()), "ally_dx sin objetivo");
    assert_eq!(sa.get(15), Some(&"0".to_string()), "ally_dy sin objetivo");

    // B: con Stranger@7,4 → enemy_dist 3, dirección +x.
    let (code_b, stdout_b, stderr_b) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Scout@4,4",
        "--place",
        "Stranger@7,4",
    ]);
    assert_eq!(code_b, 0, "exit 0, stderr: {stderr_b}");
    assert!(stderr_b.is_empty(), "stderr vacío, got: {stderr_b}");

    let sb = says_lines(&stdout_b, 0, "Scout", 4, 4);
    assert_eq!(
        sb.get(16),
        Some(&"3".to_string()),
        "enemy_dist con objetivo"
    );
    assert_eq!(
        sb.get(17),
        Some(&"1".to_string()),
        "enemy_dx hacia el objetivo"
    );
    assert_eq!(
        sb.get(18),
        Some(&"0".to_string()),
        "enemy_dy sin desplazamiento en y"
    );
}

// ---------------------------------------------------------------------------
// AC5: el más cercano es estable entre turnos ante empates.
//
// Scout@4,4 + Stranger@7,4 + Stranger@1,4 (ticks 0 y 1):
//   16 enemy_dist=3  17 enemy_dx=1  18 enemy_dy=0  (mismo objetivo en ambos ticks)
//
#[test]
fn cli_nearest_is_stable_on_ties() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "2",
        "--every",
        "1",
        "--place",
        "Scout@4,4",
        "--place",
        "Stranger@7,4",
        "--place",
        "Stranger@1,4",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    let s0 = says_lines(&stdout, 0, "Scout", 4, 4);
    let s1 = says_lines(&stdout, 1, "Scout", 4, 4);

    assert_eq!(s0.get(16), Some(&"3".to_string()), "enemy_dist en tick 0");
    assert_eq!(s0.get(17), Some(&"1".to_string()), "enemy_dx en tick 0");
    assert_eq!(s0.get(18), Some(&"0".to_string()), "enemy_dy en tick 0");

    assert_eq!(
        s1.get(16),
        Some(&"3".to_string()),
        "enemy_dist estable en tick 1"
    );
    assert_eq!(
        s1.get(17),
        Some(&"1".to_string()),
        "enemy_dx estable en tick 1"
    );
    assert_eq!(
        s1.get(18),
        Some(&"0".to_string()),
        "enemy_dy estable en tick 1"
    );
}

// ---------------------------------------------------------------------------
// AC6: `see` devuelve las cinco constantes.
//
// scout.ins emite (tick 0):
//   Caso A (Scout@4,4 + Stranger@5,5):
//     19 see(4,4)=2(ALLY)  20 see(4,5)=1(OBJECT)  21 see(5,5)=3(ENEMY)
//     22 see(0,0)=0(GROUND)  23 see(8,8)=0(GROUND)  24 see(-1,4)=1(OBJECT borde)
//   Caso B (Myope@4,4, myope.ins):
//     0 see(4,4)=2(ALLY)  1 see(8,8)=-1(NONE fuera de radio)
//
#[test]
fn cli_see_returns_the_five_constants() {
    // Scout@4,4 + Stranger@5,5: ALLY, OBJECT, ENEMY, GROUND, OBJECT(borde).
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Scout@4,4",
        "--place",
        "Stranger@5,5",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    let s = says_lines(&stdout, 0, "Scout", 4, 4);
    assert_eq!(
        s.get(19),
        Some(&"2".to_string()),
        "see(4,4)=ALLY propia casilla"
    );
    assert_eq!(s.get(20), Some(&"1".to_string()), "see(4,5)=OBJECT cristal");
    assert_eq!(s.get(21), Some(&"3".to_string()), "see(5,5)=ENEMY Stranger");
    assert_eq!(s.get(22), Some(&"0".to_string()), "see(0,0)=GROUND hierba");
    assert_eq!(
        s.get(24),
        Some(&"1".to_string()),
        "see(-1,4)=OBJECT borde fuera mapa"
    );

    // Myope@4,4 visión 1: (8,8) está fuera del radio → NONE (-1).
    let (code2, stdout2, stderr2) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Myope@4,4",
    ]);
    assert_eq!(code2, 0, "exit 0, stderr: {stderr2}");
    assert!(stderr2.is_empty(), "stderr vacío, got: {stderr2}");

    let m = says_lines(&stdout2, 0, "Myope", 4, 4);
    assert_eq!(
        m.first(),
        Some(&"2".to_string()),
        "see(4,4)=ALLY propia casilla"
    );
    assert_eq!(
        m.get(1),
        Some(&"-1".to_string()),
        "see(8,8)=NONE fuera de radio"
    );
}

// ---------------------------------------------------------------------------
// AC7: `name` devuelve especie, objeto, terreno o vacío.
//
// companion.ins emite (tick 0, Companion@4,4, vision 3):
//   0 name(4,4)="Companion"  1 name(4,5)="Crystal"  2 name(3,3)="Grass"
//   3 name(0,0)="" (fuera de radio, dist 4 > 3)
//
// myope.ins emite (tick 0, Myope@4,4, vision 1):
//   3 name(0,0)="" (fuera de radio)  4 name(9,0)="" (fuera del mapa)
//
#[test]
fn cli_name_returns_species_object_terrain_or_empty() {
    // Companion@4,4 visión 3.
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Companion@4,4",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    let c = says_lines(&stdout, 0, "Companion", 4, 4);
    assert_eq!(
        c.first(),
        Some(&"Companion".to_string()),
        "name(4,4)=especie"
    );
    assert_eq!(c.get(1), Some(&"Crystal".to_string()), "name(4,5)=cristal");
    assert_eq!(c.get(2), Some(&"Grass".to_string()), "name(3,3)=hierba");
    assert_eq!(
        c.get(3),
        Some(&"".to_string()),
        "name(0,0)=vacío fuera de radio"
    );

    // Myope@4,4 visión 1: fuera del radio y fuera del mapa devuelven "".
    let (code2, stdout2, stderr2) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Myope@4,4",
    ]);
    assert_eq!(code2, 0, "exit 0, stderr: {stderr2}");
    assert!(stderr2.is_empty(), "stderr vacío, got: {stderr2}");

    let m = says_lines(&stdout2, 0, "Myope", 4, 4);
    assert_eq!(
        m.get(3),
        Some(&"".to_string()),
        "name(0,0)=vacío fuera de radio"
    );
    assert_eq!(
        m.get(4),
        Some(&"".to_string()),
        "name(9,0)=vacío fuera del mapa"
    );
}

// ---------------------------------------------------------------------------
// AC8: `random` está en rango 0..=99 y avanza entre lecturas (estadístico).
//
// scout.ins emite (tick 0, Scout@4,4):
//   27 random=valor1  28 random=valor2
//
#[test]
fn cli_random_is_in_range_and_advances() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &bank_dir(),
        "--map",
        "Mixed",
        "--seed",
        "1",
        "--ticks",
        "1",
        "--every",
        "1",
        "--place",
        "Scout@4,4",
    ]);
    assert_eq!(code, 0, "exit 0, stderr: {stderr}");
    assert!(stderr.is_empty(), "stderr vacío, got: {stderr}");

    let s = says_lines(&stdout, 0, "Scout", 4, 4);
    let r1 = s
        .get(27)
        .expect("primera lectura de random")
        .parse::<i64>()
        .expect("random debe ser entero");
    let r2 = s
        .get(28)
        .expect("segunda lectura de random")
        .parse::<i64>()
        .expect("random debe ser entero");
    assert!((0..=99).contains(&r1), "random1 en rango, got {r1}");
    assert!((0..=99).contains(&r2), "random2 en rango, got {r2}");
    assert_ne!(r1, r2, "dos lecturas de random son distintas");
}
