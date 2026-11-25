// Pruebas de escenario F6-05 (núcleo): valores exactos de expresiones sobre el
// estado interno del mundo, cargando las criaturas de `examples/`.

use super::BankWorld;
use instinct_core::event::Event;

/// Devuelve los textos dichos por la criatura de `species` en el siguiente
/// tick, en orden de emisión. Cada prueba construye un mundo nuevo y avanza
/// exactamente un tick.
fn said_texts(bw: &mut BankWorld, species: &str) -> Vec<String> {
    bw.creature_ids
        .iter()
        .find(|(name, _)| name == species)
        .expect("especie no encontrada en la colocación");
    let events = bw.step_tick();
    events
        .into_iter()
        .filter_map(|e| match e {
            Event::Said {
                species: s, text, ..
            } if s == species => Some(text),
            _ => None,
        })
        .collect()
}

// AC1: precedencia del enunciado (paréntesis, unario, *, /, %, +, -, comparaciones, and, or).
#[test]
fn precedence_matches_enunciado() {
    let mut bw = BankWorld::from_examples("Arena", &[("Calculator", 2, 2)]);
    let texts = said_texts(&mut bw, "Calculator");
    assert_eq!(
        texts,
        vec!["14", "20", "4", "-2", "-5", "3", "1", "1", "0", "0", "1", "1"],
        "valores de precedencia de la calculadora"
    );
}

// AC2: división y resto enteros con suelo y signo del divisor.
#[test]
fn division_and_remainder_floor() {
    let mut bw = BankWorld::from_examples("Arena", &[("Divider", 2, 2)]);
    let texts = said_texts(&mut bw, "Divider");
    assert_eq!(
        texts,
        vec!["3", "-4", "-4", "3", "1", "2", "-2", "-1"],
        "cociente y resto con signos mixtos"
    );
}

// AC3: verdad de números y textos (operadores lógicos sobre enteros y
// textos; comparaciones devuelven 1 o 0).
#[test]
fn truthiness_of_numbers_and_texts() {
    let mut bw = BankWorld::from_examples("Arena", &[("Truths", 2, 2)]);
    let texts = said_texts(&mut bw, "Truths");
    assert_eq!(
        texts,
        vec!["1", "0", "0", "1", "0", "0", "1", "1", "0"],
        "not sobre 0/1/-5/\" \"/\"hola\", \"\" or 0, \"hola\" and 1, y comparaciones"
    );
}

/// Devuelve true si no hay criatura viva en la celda (x, y).
fn cell_free(bw: &BankWorld, x: usize, y: usize) -> bool {
    !bw.sim
        .world
        .creatures
        .iter()
        .flatten()
        .any(|c| c.x == x && c.y == y)
}

/// Una criatura trampa debe emitir `RuntimeError` en la línea 7, morir con
/// causa `"error"`, ser eliminada del mapa (`memento` es `None`) y dejar su
/// celda libre.
fn trap_dies_by_error(species: &str, x: usize, y: usize) {
    let mut bw = BankWorld::from_examples("Arena", &[(species, x, y)]);
    let events = bw.step_tick();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::RuntimeError { line: 7, .. })),
        "{species} debe emitir RuntimeError en la línea 7"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Died { cause, .. } if cause == "error")),
        "{species} debe morir con causa de error"
    );
    assert!(
        bw.memento(species).is_none(),
        "{species} debe quedar eliminada del mapa"
    );
    assert!(
        cell_free(&bw, x, y),
        "la celda de {species} debe quedar libre"
    );
}

// AC8: las criaturas trampa mueren por error de ejecución en la línea 7,
// solo ellas mismas, y quedan fuera del mapa.
#[test]
fn text_order_comparison_dies() {
    trap_dies_by_error("TextOrder", 2, 2);
}

#[test]
fn mixed_types_die_with_line() {
    trap_dies_by_error("Mixer", 2, 2);
}

#[test]
fn div_zero_and_unassigned_die() {
    trap_dies_by_error("DividerZero", 2, 2);
    trap_dies_by_error("Forgetful", 2, 2);
}

#[test]
fn text_argument_and_bad_direction_die() {
    trap_dies_by_error("Wordy", 2, 2);
    trap_dies_by_error("OutOfRange", 2, 2);
}

#[test]
fn runtime_error_kills_only_its_creature() {
    let mut bw = BankWorld::from_examples("Arena", &[("Wordy", 2, 2), ("Victim", 6, 2)]);
    let events = bw.step_tick();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::RuntimeError { line: 7, .. })),
        "Wordy debe emitir RuntimeError en la línea 7"
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Died { cause, .. } if cause == "error")),
        "Wordy debe morir con causa de error"
    );
    assert!(
        bw.memento("Wordy").is_none(),
        "Wordy debe estar eliminada del mapa"
    );
    assert!(
        bw.memento("Victim").is_some(),
        "Victim debe seguir viva junto a Wordy"
    );
}
