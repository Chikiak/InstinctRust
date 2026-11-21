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

// AC8 (núcleo): las variables persisten de un turno a otro y el pc se
// reanuda en start al llegar al final del archivo.
#[test]
fn variables_persist_across_turns() {
    let mut bw = BankWorld::from_examples("Arena", &[("Counter", 2, 2)]);
    // Tick 0: marked = tick (0), say(0), goto start, ... hasta el límite de líneas.
    bw.step_tick();
    let m0 = bw.memento("Counter").expect("Counter vivo tras tick 0");
    assert_eq!(
        m0.health, 19,
        "Counter: 20 - 1 fin de tick = 19"
    );
    assert_eq!(
        m0.variables.get("marked"),
        Some(&instinct_core::value::Value::Int(0)),
        "marcada = 0 (tick durante el turno de tick 0)"
    );
    // Tick 1: continúa desde el pc donde se quedó, marcada se reasigna a 1.
    bw.step_tick();
    let m1 = bw.memento("Counter").expect("Counter vivo tras tick 1");
    assert_eq!(
        m1.variables.get("marked"),
        Some(&instinct_core::value::Value::Int(1)),
        "marcada = 1 (tick durante el turno de tick 1, variable persistió)"
    );
}

// AC7 (núcleo): ciclo sin acción se corta en el límite de líneas por turno;
// la simulación sigue y la criatura sobrevive.
#[test]
fn line_limit_caps_counter_per_turn() {
    let mut bw = BankWorld::from_examples("Arena", &[("Spinner", 2, 2)]);
    bw.step_tick();
    let m = bw.memento("Spinner").expect("Spinner vivo tras el tick");
    // Ciclo de 3 líneas contando la etiqueta `start:` (n = 1, goto start);
    // 33 ciclos = 99 líneas y la línea 100 (la etiqueta) alcanza el límite:
    // pc apunta a la primera no ejecutada, la línea 1.
    assert_eq!(m.pc, 1, "Spinner: pc=1 tras 100 líneas del ciclo");
    assert_eq!(m.health, 19, "Spinner: 20 - 1 fin de tick = 19");
    assert_eq!(m.age, 1, "Spinner: edad 1 tras fin de tick");
    // La simulación sigue: Spinner sigue viva y puede seguir ejecutando.
    bw.step_tick();
    let m2 = bw.memento("Spinner").expect("Spinner sigue viva en tick 2");
    assert_eq!(m2.health, 18, "Spinner: 19 - 1 fin de tick = 18");
}

// AC5 (núcleo): reproduce aborta con destino ocupado por criatura,
// no nace nadie y la madre sigue viva (sin costo de salud por el aborto).
#[test]
fn reproduce_aborts_cell_occupied_by_creature() {
    // Mother@2,2 con reproduce(0, 1, 5): destino (2,3) ocupado por Victim.
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 2, 2), ("Victim", 2, 3)]);
    bw.step_tick();
    let mother = bw.memento("Mother").expect("Mother viva tras el aborto");
    assert_eq!(
        mother.health, 19,
        "Mother: 20 - 1 fin de tick, reproduce abortó sin costo"
    );
    assert_eq!(mother.x, 2, "Mother sigue en (2,2)");
    assert_eq!(mother.y, 2, "Mother sigue en (2,2)");
    // La casilla destino sigue ocupada por la Victim y no nace cría.
    let victim_id = bw
        .creature_ids
        .iter()
        .find(|(name, _)| name == &"Victim")
        .map(|(_, id)| *id)
        .expect("Victim colocada");
    assert_eq!(
        bw.find_child_at(2, 3, &[]),
        Some(victim_id),
        "(2,3) sigue ocupada por la Victim, sin cría"
    );
}

// AC5 (núcleo): reproduce aborta con destino que tiene una roca.
#[test]
fn reproduce_aborts_cell_has_rock() {
    // Mother@4,3 con reproduce(0, 1, 5): destino (4,4) es la roca de Arena.
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 4, 3)]);
    bw.step_tick();
    let mother = bw.memento("Mother").expect("Mother viva tras el aborto");
    assert_eq!(
        mother.health, 19,
        "Mother: 20 - 1 fin de tick, reproduce abortó sin costo"
    );
    // No nace cría en (4,4): solo hay una criatura viva (la madre).
    assert_eq!(
        bw.sim.world.live_creature_ids().len(),
        1,
        "sigue viva solo la Mother"
    );
    assert!(
        bw.find_child_at(4, 4, &[]).is_none(),
        "(4,4) tiene la roca, no nace cría"
    );
}

// AC5 (núcleo): reproduce aborta con destino fuera del mapa.
#[test]
fn reproduce_aborts_out_of_map() {
    // Mother@2,8 con reproduce(0, 1, 5): destino (2,9) fuera (filas 0..8).
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 2, 8)]);
    bw.step_tick();
    let mother = bw.memento("Mother").expect("Mother viva tras el aborto");
    assert_eq!(
        mother.health, 19,
        "Mother: 20 - 1 fin de tick, reproduce abortó sin costo"
    );
    assert_eq!(
        bw.sim.world.live_creature_ids().len(),
        1,
        "live_creature_ids == 1: solo la Mother"
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

// AC6 (núcleo): una criatura que muere antes de su turno no juega:
// tras step_ticks(3) PoorMother (hp 3) ha muerto y no queda nadie.
#[test]
fn creature_dead_before_turn_does_not_play() {
    let mut bw = BankWorld::from_examples("Arena", &[("PoorMother", 2, 2)]);
    let mother_id = bw
        .creature_ids
        .iter()
        .find(|(name, _)| name == &"PoorMother")
        .map(|(_, id)| *id)
        .expect("PoorMother colocada");
    // hp 3: tick 0 -> 2, tick 1 -> 1, tick 2 -> 0 (muere al fin de tick).
    bw.step_ticks(3);
    assert!(
        bw.sim.world.creatures[mother_id.0].is_none(),
        "PoorMother slot vacía tras morir"
    );
    assert_eq!(
        bw.sim.world.live_creature_ids().len(),
        0,
        "live_creature_ids vacía: nadie juega tras la muerte"
    );
}
