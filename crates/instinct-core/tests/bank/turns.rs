//! Pruebas de escenario F6-03 (núcleo): reproducción, aborto natural y ciclo
//! de turnos sobre el estado interno del mundo.

use super::BankWorld;

// AC1 (núcleo): reproduce con casilla libre y salud suficiente crea cría
// con edad 0, vida pedida y sin variables.
// Convención: tras 1 step_tick, end_of_tick ya aplicó +1 edad y -1 salud,
// así que la cría muestra edad 1 y health 4 (nació con edad 0 y health 5).
#[test]
fn birth_has_age_zero_and_no_variables() {
    // Mother: health 20, reproduce(0, 1, 5) → cría en (2,3) con health 5.
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 2, 2)]);
    bw.step_tick();
    // La madre sigue en (2,2).
    let mother = bw.memento("Mother").expect("Mother viva");
    assert_eq!(mother.x, 2, "Mother x tras el nacimiento");
    assert_eq!(mother.y, 2, "Mother y tras el nacimiento");
    // Leer al recién nacido directamente del almacén (no está en creature_ids).
    let child_id = bw
        .find_child_at(2, 3, &[])
        .expect("la cría debe estar en (2,3)");
    let child = bw.sim.world.creatures[child_id.0].as_ref().expect("child");
    let m = child.memento();
    assert_eq!(
        m.age, 1,
        "AC1: cría tras 1 step_tick: nació con edad 0, +1 fin de tick = 1"
    );
    assert_eq!(
        m.health, 4,
        "AC1: cría tras 1 step_tick: nació con health 5, -1 fin de tick = 4"
    );
    assert!(
        m.variables.is_empty(),
        "AC1: cría nace sin variables, got {:?}",
        m.variables
    );
}

// AC2 (núcleo): la madre pierde la vida que transfiere y la cría nace con esa vida.
#[test]
fn birth_transfers_health() {
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 2, 2)]);
    bw.step_tick();
    let mother = bw.memento("Mother").expect("Mother viva");
    // 20 - 5 (reproduce) - 1 (fin de tick) = 14
    assert_eq!(
        mother.health, 14,
        "AC2: Mother: 20 - 5 (transfiere) - 1 (fin de tick) = 14"
    );
    let child_id = bw
        .find_child_at(2, 3, &[])
        .expect("la cría debe estar en (2,3)");
    let child = bw.sim.world.creatures[child_id.0].as_ref().expect("child");
    let m = child.memento();
    // 5 (nace) - 1 (fin de tick) = 4
    assert_eq!(
        m.health, 4,
        "AC2: cría: 5 (vida pedida) - 1 (fin de tick) = 4"
    );
}

// AC5 (núcleo): reproduce aborta por salud insuficiente, no nace nadie,
// la madre sigue viva (solo paga el fin de tick).
#[test]
fn reproduce_aborts_insufficient_health() {
    // PoorMother: health 3, reproduce(0, 1, 5). 3 <= 5 → aborto natural.
    let mut bw = BankWorld::from_examples("Arena", &[("PoorMother", 2, 2)]);
    bw.step_tick();
    let mother = bw.memento("PoorMother").expect("PoorMother viva tras el aborto");
    assert_eq!(
        mother.health, 2,
        "PoorMother: 3 - 1 (fin de tick) = 2, no gastó vida en reproduce"
    );
    assert_eq!(mother.age, 1, "PoorMother envejece 1 al fin de tick");
    // No cría en (2, 3): la casilla destino sigue vacía.
    assert!(
        bw.find_child_at(2, 3, &[]).is_none(),
        "no nace cría por salud insuficiente"
    );
}
