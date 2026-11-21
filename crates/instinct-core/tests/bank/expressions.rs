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

// AC3: verdad de números y textos (comparaciones devuelven 1 o 0).
#[test]
fn truthiness_of_numbers_and_texts() {
    let mut bw = BankWorld::from_examples("Arena", &[("Truths", 2, 2)]);
    let texts = said_texts(&mut bw, "Truths");
    assert_eq!(
        texts,
        vec!["0", "1", "", "hola", "1", "0"],
        "truthiness de 0, 1, \"\", \"hola\" y comparaciones"
    );
}
