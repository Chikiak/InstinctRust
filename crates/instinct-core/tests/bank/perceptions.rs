// Pruebas de escenario F6-04 (núcleo): percepciones propias, de mundo y de
// cercanía, `see` y `name` sobre el banco `examples/`.
//
// Usa `BankWorld::from_examples` + `eval()` para leer valores exactos de las
// percepciones y de `see`/`name`. El mapa `mixed.map` tiene 9x9: hierba ('.')
// en casi todo, columna de arena ('s') en x=4, roca ('#') en (3,5) y cristal
// ('*') en (4,5). Scout tiene vision 6 (según su cabecera); Myope vision 1;
// Companion y Stranger vision 3.

use super::BankWorld;
use instinct_core::value::Value;

fn unw_int(v: Value) -> i64 {
    match v {
        Value::Int(n) => n,
        _ => panic!("esperado int, got {v:?}"),
    }
}

fn unw_txt(v: Value) -> String {
    match v {
        Value::Text(t) => t,
        _ => panic!("esperado texto, got {v:?}"),
    }
}

fn scout_id(bw: &BankWorld) -> instinct_core::world::CreatureId {
    bw.creature_ids
        .iter()
        .find(|(n, _)| n == "Scout")
        .map(|(_, id)| *id)
        .expect("Scout")
}

/// AC1: percepciones propias coinciden con la colocación y la cabecera.
#[test]
fn self_perceptions_match_placement() {
    let mut bw = BankWorld::from_examples("Mixed", &[("Scout", 2, 2)]);
    bw.step_tick();
    let id = scout_id(&bw);
    assert_eq!(
        unw_int(bw.eval(id, "health").unwrap()),
        19,
        "health tras fin de tick"
    );
    assert_eq!(unw_int(bw.eval(id, "age").unwrap()), 1, "age tras un tick");
    assert_eq!(unw_int(bw.eval(id, "x").unwrap()), 2, "x");
    assert_eq!(unw_int(bw.eval(id, "y").unwrap()), 2, "y");
    assert_eq!(unw_txt(bw.eval(id, "species").unwrap()), "Scout");
    assert_eq!(unw_txt(bw.eval(id, "faction").unwrap()), "explorers");
    assert_eq!(
        unw_txt(bw.eval(id, "terrain_here").unwrap()),
        "Grass",
        "(2,2) es hierba"
    );
    assert_eq!(
        unw_int(bw.eval(id, "reserve_here").unwrap()),
        100,
        "reserva hierba"
    );
    assert_eq!(unw_int(bw.eval(id, "tick").unwrap()), 1, "tick");
    assert_eq!(unw_int(bw.eval(id, "width").unwrap()), 9, "ancho");
    assert_eq!(unw_int(bw.eval(id, "height").unwrap()), 9, "alto");
    assert_eq!(
        unw_int(bw.eval(id, "total_creatures").unwrap()),
        0,
        "solo el explorador, no se cuenta"
    );
    assert_eq!(
        unw_int(bw.eval(id, "total_allies").unwrap()),
        0,
        "sin aliados"
    );
    assert_eq!(
        unw_int(bw.eval(id, "total_enemies").unwrap()),
        0,
        "sin enemigos"
    );
    assert_eq!(
        unw_int(bw.eval(id, "vision").unwrap()),
        6,
        "vision del explorador"
    );
}

/// AC2: totales del mundo excluyen a la propia criatura.
#[test]
fn world_perceptions_count_others() {
    let mut bw = BankWorld::from_examples(
        "Mixed",
        &[("Scout", 2, 2), ("Companion", 5, 5), ("Stranger", 7, 7)],
    );
    bw.step_tick();
    let id = scout_id(&bw);
    assert_eq!(unw_int(bw.eval(id, "width").unwrap()), 9);
    assert_eq!(unw_int(bw.eval(id, "height").unwrap()), 9);
    assert_eq!(unw_int(bw.eval(id, "tick").unwrap()), 1);
    assert_eq!(unw_int(bw.eval(id, "total_creatures").unwrap()), 2);
    assert_eq!(unw_int(bw.eval(id, "total_allies").unwrap()), 1);
    assert_eq!(unw_int(bw.eval(id, "total_enemies").unwrap()), 1);
}

/// AC3: conteos de cercanía excluyen la casilla propia.
/// Myope@5,4 con vision 1: cristal en (4,5) dist 1 (1 objeto), roca en (3,5)
/// dist 2 (fuera de radio). Companion@5,5 dist 1 (1 aliado), Stranger@5,3
/// dist 1 (1 enemigo).
#[test]
fn near_counts_exclude_own_cell() {
    let mut bw = BankWorld::from_examples(
        "Mixed",
        &[("Myope", 5, 4), ("Companion", 5, 5), ("Stranger", 5, 3)],
    );
    bw.step_tick();
    let myope_id = bw
        .creature_ids
        .iter()
        .find(|(n, _)| n == "Myope")
        .map(|(_, id)| *id)
        .expect("Myope");
    assert_eq!(
        unw_int(bw.eval(myope_id, "allies_near").unwrap()),
        1,
        "Companion en (5,5) dist 1"
    );
    assert_eq!(
        unw_int(bw.eval(myope_id, "enemies_near").unwrap()),
        1,
        "Stranger en (5,3) dist 1"
    );
    assert_eq!(
        unw_int(bw.eval(myope_id, "objects_near").unwrap()),
        1,
        "cristal en (4,5) dist 1; roca en (3,5) fuera de radio"
    );
}

/// AC4: más cercano sin y con objetivo.
#[test]
fn nearest_directions_and_distance() {
    // Sin objetivo visible.
    let mut bw = BankWorld::from_examples("Mixed", &[("Scout", 4, 4)]);
    bw.step_tick();
    let id = scout_id(&bw);
    assert_eq!(unw_int(bw.eval(id, "enemy_dist").unwrap()), -1);
    assert_eq!(unw_int(bw.eval(id, "enemy_dx").unwrap()), 0);
    assert_eq!(unw_int(bw.eval(id, "enemy_dy").unwrap()), 0);
    assert_eq!(unw_int(bw.eval(id, "ally_dist").unwrap()), -1);
    assert_eq!(unw_int(bw.eval(id, "ally_dx").unwrap()), 0);
    assert_eq!(unw_int(bw.eval(id, "ally_dy").unwrap()), 0);

    // Sin objetos visibles: Myope@0,0 con visión 1 no ve nada → object_dist -1.
    let mut bw2 = BankWorld::from_examples("Mixed", &[("Myope", 0, 0)]);
    bw2.step_tick();
    let myope_id = bw2
        .creature_ids
        .iter()
        .find(|(n, _)| n == "Myope")
        .map(|(_, id)| *id)
        .expect("Myope");
    assert_eq!(
        unw_int(bw2.eval(myope_id, "object_dist").unwrap()),
        -1,
        "sin objetos visibles"
    );
    assert_eq!(
        unw_int(bw2.eval(myope_id, "object_dx").unwrap()),
        0,
        "object_dx sin objetos"
    );
    assert_eq!(
        unw_int(bw2.eval(myope_id, "object_dy").unwrap()),
        0,
        "object_dy sin objetos"
    );

    // Con objetivo: Stranger@7,4 (arena, dist 3, dirección +x).
    let mut bw = BankWorld::from_examples("Mixed", &[("Scout", 4, 4), ("Stranger", 7, 4)]);
    bw.step_tick();
    let id = scout_id(&bw);
    assert_eq!(unw_int(bw.eval(id, "enemy_dist").unwrap()), 3);
    assert_eq!(unw_int(bw.eval(id, "enemy_dx").unwrap()), 1);
    assert_eq!(unw_int(bw.eval(id, "enemy_dy").unwrap()), 0);
    assert_eq!(
        unw_int(bw.eval(id, "ally_dist").unwrap()),
        -1,
        "sin aliados con objetivo"
    );
    assert_eq!(
        unw_int(bw.eval(id, "object_dist").unwrap()),
        1,
        "objeto más cercano (cristal en (4,5))"
    );
    assert_eq!(
        unw_int(bw.eval(id, "object_dx").unwrap()),
        -1,
        "object_dx hacia objeto más cercano (roca en (3,5); empate con cristal en (4,5))"
    );
    assert_eq!(
        unw_int(bw.eval(id, "object_dy").unwrap()),
        1,
        "object_dy hacia objeto más cercano (roca en (3,5); empate con cristal en (4,5))"
    );
}

/// AC5: empate en más cercano es estable entre turnos.
#[test]
fn nearest_is_stable_on_ties() {
    let mut bw = BankWorld::from_examples(
        "Mixed",
        &[("Scout", 4, 4), ("Stranger", 7, 4), ("Stranger", 1, 4)],
    );
    bw.step_tick();
    let id = scout_id(&bw);
    let d1 = unw_int(bw.eval(id, "enemy_dist").unwrap());
    let dx1 = unw_int(bw.eval(id, "enemy_dx").unwrap());
    let dy1 = unw_int(bw.eval(id, "enemy_dy").unwrap());
    bw.step_tick();
    let d2 = unw_int(bw.eval(id, "enemy_dist").unwrap());
    let dx2 = unw_int(bw.eval(id, "enemy_dx").unwrap());
    let dy2 = unw_int(bw.eval(id, "enemy_dy").unwrap());
    assert_eq!(d1, 3, "enemy_dist en primer tick");
    assert_eq!(d2, 3, "enemy_dist estable en segundo tick");
    assert_eq!(d1, d2, "mismo objetivo en empate");
    assert_eq!(dx1, dx2, "enemy_dx estable en segundo tick");
    assert_eq!(dy1, dy2, "enemy_dy estable en segundo tick");
}

/// AC6: `see` devuelve las cinco constantes.
/// Scout tiene vision 6, así que (0,0) está dentro del radio (dist 4 <= 6).
/// Para NONE (fuera de radio) usa Myope con vision 1.
#[test]
fn see_returns_the_five_constants() {
    let mut bw = BankWorld::from_examples("Mixed", &[("Scout", 4, 4), ("Stranger", 5, 5)]);
    bw.step_tick();
    let id = scout_id(&bw);
    assert_eq!(
        unw_int(bw.eval(id, "see(4,4)").unwrap()),
        2,
        "ALLY propia casilla"
    );
    assert_eq!(
        unw_int(bw.eval(id, "see(4,5)").unwrap()),
        1,
        "OBJECT cristal en (4,5)"
    );
    assert_eq!(
        unw_int(bw.eval(id, "see(3,5)").unwrap()),
        1,
        "OBJECT roca en (3,5)"
    );
    assert_eq!(
        unw_int(bw.eval(id, "see(3,3)").unwrap()),
        0,
        "GROUND hierba en (3,3)"
    );
    assert_eq!(
        unw_int(bw.eval(id, "see(-1,4)").unwrap()),
        1,
        "OBJECT borde fuera mapa"
    );
    assert_eq!(
        unw_int(bw.eval(id, "see(5,5)").unwrap()),
        3,
        "ENEMY Stranger en (5,5)"
    );

    // Scenario S30: Companion@5,5 → see(5,5) = ALLY (2).
    let mut bw3 = BankWorld::from_examples("Mixed", &[("Scout", 4, 4), ("Companion", 5, 5)]);
    bw3.step_tick();
    let id3 = scout_id(&bw3);
    assert_eq!(
        unw_int(bw3.eval(id3, "see(5,5)").unwrap()),
        2,
        "ALLY compañero en (5,5) scenario S30"
    );

    // Myope vision 1: (0,0) está a dist 4, fuera de radio → NONE.
    let mut bw2 = BankWorld::from_examples("Mixed", &[("Myope", 4, 4)]);
    bw2.step_tick();
    let myope_id = bw2
        .creature_ids
        .iter()
        .find(|(n, _)| n == "Myope")
        .map(|(_, id)| *id)
        .expect("Myope");
    assert_eq!(
        unw_int(bw2.eval(myope_id, "see(0,0)").unwrap()),
        -1,
        "NONE fuera de radio (vision 1, dist 4)"
    );
}

/// AC7: `name` devuelve especie, objeto, terreno o vacío.
/// Companion@4,4 vision 3: (2,2) está dentro del radio (dist 2).
/// Myope@4,4 vision 1: (0,0) fuera de radio, (9,0) fuera del mapa.
#[test]
fn name_returns_species_object_terrain_or_empty() {
    let mut bw = BankWorld::from_examples("Mixed", &[("Companion", 4, 4), ("Stranger", 5, 5)]);
    bw.step_tick();
    let comp_id = bw
        .creature_ids
        .iter()
        .find(|(n, _)| n == "Companion")
        .map(|(_, id)| *id)
        .expect("Companion");
    assert_eq!(
        unw_txt(bw.eval(comp_id, "name(4,4)").unwrap()),
        "Companion",
        "especie en casilla propia"
    );
    assert_eq!(
        unw_txt(bw.eval(comp_id, "name(4,5)").unwrap()),
        "Crystal",
        "objeto cristal en (4,5)"
    );
    assert_eq!(
        unw_txt(bw.eval(comp_id, "name(3,3)").unwrap()),
        "Grass",
        "terreno hierba en (3,3)"
    );
    assert_eq!(
        unw_txt(bw.eval(comp_id, "name(2,2)").unwrap()),
        "Grass",
        "terreno hierba en (2,2) dentro de radio"
    );
    assert_eq!(
        unw_txt(bw.eval(comp_id, "name(4,2)").unwrap()),
        "Sand",
        "terreno arena en (4,2)"
    );
    assert_eq!(
        unw_txt(bw.eval(comp_id, "name(3,5)").unwrap()),
        "Rock",
        "objeto roca en (3,5)"
    );
    assert_eq!(
        unw_txt(bw.eval(comp_id, "name(5,5)").unwrap()),
        "Stranger",
        "especie desconocida en (5,5)"
    );

    // Myope vision 1: fuera de radio y fuera de mapa devuelven "".
    let mut bw2 = BankWorld::from_examples("Mixed", &[("Myope", 4, 4)]);
    bw2.step_tick();
    let myope_id = bw2
        .creature_ids
        .iter()
        .find(|(n, _)| n == "Myope")
        .map(|(_, id)| *id)
        .expect("Myope");
    assert_eq!(
        unw_txt(bw2.eval(myope_id, "name(0,0)").unwrap()),
        "",
        "fuera de radio (vision 1, dist 4)"
    );
    assert_eq!(
        unw_txt(bw2.eval(myope_id, "name(9,0)").unwrap()),
        "",
        "fuera del mapa"
    );
}

/// AC8: `random` está en rango y avanza entre lecturas.
#[test]
fn random_is_in_range_and_advances() {
    let mut bw = BankWorld::from_examples("Mixed", &[("Scout", 4, 4)]);
    bw.sim.set_rng_seed(42);
    bw.step_tick();
    let id = scout_id(&bw);
    let r1 = unw_int(bw.eval(id, "random").unwrap());
    let r2 = unw_int(bw.eval(id, "random").unwrap());
    assert!((0..=99).contains(&r1), "random1 en rango, got {r1}");
    assert!((0..=99).contains(&r2), "random2 en rango, got {r2}");
    assert_ne!(r1, r2, "dos lecturas de random son distintas");
}
