//! Pruebas de escenario F6-03 (núcleo): reproducción, aborto natural y ciclo
//! de turnos sobre el estado interno del mundo.

use super::BankWorld;

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
