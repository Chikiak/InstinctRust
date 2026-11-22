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
// reanuda en start al llegar al final del archivo. `marked` se asigna en
// el turno 0 y se lee (sin reasignar) en el turno 1.
#[test]
fn variables_persist_across_turns() {
    let mut bw = BankWorld::from_examples("Arena", &[("Counter", 2, 2)]);
    // Tick 0: marked = tick (0), wait(1) termina el turno; marked = 0.
    bw.step_tick();
    let m0 = bw.memento("Counter").expect("Counter vivo tras tick 0");
    assert_eq!(m0.health, 19, "Counter: 20 - 1 fin de tick = 19");
    assert_eq!(
        m0.variables.get("marked"),
        Some(&instinct_core::value::Value::Int(0)),
        "marcada = 0 (tick durante el turno de tick 0)"
    );
    // Tick 1: salta la asignación (tick > 0), dice marked = 0 sin reasignarla.
    let events1 = bw.step_tick();
    let m1 = bw.memento("Counter").expect("Counter vivo tras tick 1");
    assert_eq!(
        m1.variables.get("marked"),
        Some(&instinct_core::value::Value::Int(0)),
        "marcada sigue siendo 0 en tick 1 (no reasignada antes de leerla)"
    );
    // El evento Said en tick 1 lleva el valor guardado en el turno anterior.
    assert!(
        events1.iter().any(|e| matches!(
            e,
            instinct_core::event::Event::Said { species, pos, text, .. }
                if species == "Counter"
                    && *pos == instinct_core::event::Pos { x: 2, y: 2 }
                    && text == "0"
        )),
        "esperaba Said '0' en tick 1, eventos: {:?}",
        events1
    );
    // Tras llegar al final del archivo, pc se reinició a start: para el
    // turno siguiente.
    assert_eq!(
        m1.pc, 0,
        "pc == 0 tras llegar al final del archivo (reanuda en start:)"
    );
}

// AC7 (núcleo): ciclo sin acción se corta en el límite de líneas por turno;
// la simulación sigue y la criatura sobrevive. El contador `n` se incrementa
// en cada iteración del bucle y se limita a 100 líneas por turno.
#[test]
fn line_limit_caps_counter_per_turn() {
    let mut bw = BankWorld::from_examples("Arena", &[("Spinner", 2, 2)]);
    // Tick 0: n = 0; bucle loop: n = n + 1; goto loop hasta el límite.
    bw.step_tick();
    let m = bw.memento("Spinner").expect("Spinner vivo tras el tick");
    assert_eq!(m.health, 19, "Spinner: 20 - 1 fin de tick = 19");
    assert_eq!(m.age, 1, "Spinner: edad 1 tras fin de tick");
    // 1 línea (n = 0) + 33 iteraciones × 3 líneas (loop:, n = n + 1, goto loop)
    // = 100 líneas, por lo que n == 33 y siempre <= 100.
    let n0 = m
        .variables
        .get("n")
        .cloned()
        .unwrap_or(instinct_core::value::Value::Int(0));
    let n0_i64 = match n0 {
        instinct_core::value::Value::Int(n) => n,
        _ => panic!("Spinner: n debe ser Int, got {:?}", n0),
    };
    assert_eq!(
        n0_i64, 33,
        "Spinner: n == 33 tras 100 líneas en tick 0, got {}",
        n0_i64
    );
    assert!(
        n0_i64 <= 100,
        "Spinner: n <= 100 (límite de líneas por turno)"
    );
    // Tick 1: continúa desde pc en `goto loop`, n vuelve a incrementarse.
    bw.step_tick();
    let m2 = bw.memento("Spinner").expect("Spinner sigue viva en tick 2");
    assert_eq!(m2.health, 18, "Spinner: 19 - 1 fin de tick = 18");
    let n1 = m2
        .variables
        .get("n")
        .cloned()
        .unwrap_or(instinct_core::value::Value::Int(0));
    let n1_i64 = match n1 {
        instinct_core::value::Value::Int(n) => n,
        _ => panic!("Spinner: n debe ser Int, got {:?}", n1),
    };
    assert!(
        n1_i64 > 33,
        "Spinner: n incrementó en el segundo turno, got {}",
        n1_i64
    );
}

// AC5 (núcleo): reproduce aborta con destino ocupado por criatura,
// no nace nadie y la madre sigue viva (sin costo de salud por el aborto).
#[test]
fn reproduce_aborts_cell_occupied_by_creature() {
    // Mother@2,2 con reproduce(0, 1, 5): destino (2,3) ocupado por Victim.
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 2, 2), ("Victim", 2, 3)]);
    let events = bw.step_tick();
    assert!(
        events.is_empty(),
        "el aborto natural no debe emitir eventos, got {:?}",
        events
    );
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
        .find(|(name, _)| name == "Victim")
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
    let events = bw.step_tick();
    assert!(
        events.is_empty(),
        "el aborto natural no debe emitir eventos, got {:?}",
        events
    );
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
    let events = bw.step_tick();
    assert!(
        events.is_empty(),
        "el aborto natural no debe emitir eventos, got {:?}",
        events
    );
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
    let events = bw.step_tick();
    assert!(
        events.is_empty(),
        "el aborto natural no debe emitir eventos, got {:?}",
        events
    );
    let mother = bw
        .memento("PoorMother")
        .expect("PoorMother viva tras el aborto");
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

// AC5 (núcleo): reproduce aborta silenciosamente en los 4 casos naturales
// (destino ocupado, roca, fuera del mapa, salud insuficiente): no nace
// nadie, la madre sigue viva y no se emite ningún evento.
#[test]
fn reproduce_aborts_silently() {
    // Caso 1: destino ocupado por criatura.
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 2, 2), ("Victim", 2, 3)]);
    let victim_id = bw
        .creature_ids
        .iter()
        .find(|(name, _)| name == "Victim")
        .map(|(_, id)| *id)
        .expect("Victim colocada");
    let events = bw.step_tick();
    assert!(events.is_empty(), "ocupado: sin eventos, got {:?}", events);
    assert_eq!(
        bw.memento("Mother").expect("madre viva").health,
        19,
        "ocupado: madre viva"
    );
    assert!(
        bw.find_child_at(2, 3, &[victim_id]).is_none(),
        "ocupado: sin cría distinta de la Victim"
    );

    // Caso 2: destino con roca.
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 4, 3)]);
    let events = bw.step_tick();
    assert!(events.is_empty(), "roca: sin eventos, got {:?}", events);
    assert_eq!(
        bw.memento("Mother").expect("madre viva").health,
        19,
        "roca: madre viva"
    );
    assert_eq!(
        bw.sim.world.live_creature_ids().len(),
        1,
        "roca: solo la madre"
    );

    // Caso 3: destino fuera del mapa.
    let mut bw = BankWorld::from_examples("Arena", &[("Mother", 2, 8)]);
    let events = bw.step_tick();
    assert!(events.is_empty(), "fuera: sin eventos, got {:?}", events);
    assert_eq!(
        bw.memento("Mother").expect("madre viva").health,
        19,
        "fuera: madre viva"
    );
    assert_eq!(
        bw.sim.world.live_creature_ids().len(),
        1,
        "fuera: solo la madre"
    );

    // Caso 4: salud insuficiente.
    let mut bw = BankWorld::from_examples("Arena", &[("PoorMother", 2, 2)]);
    let events = bw.step_tick();
    assert!(events.is_empty(), "salud: sin eventos, got {:?}", events);
    assert_eq!(
        bw.memento("PoorMother").expect("madre viva").health,
        2,
        "salud: madre viva"
    );
    assert!(bw.find_child_at(2, 3, &[]).is_none(), "salud: sin cría");
}

// AC6 (núcleo): una criatura que muere antes de su turno dentro del mismo
// tick no juega: Brawler (daño 10) mata a TalkerVictim (hp 5) en el tick 0
// y TalkerVictim no emite ningún evento de acción.
#[test]
fn creature_dead_before_turn_does_not_play() {
    // Brawler@2,2 ataca a (3,2) donde está TalkerVictim (hp 5). Con seed 1 el
    // Brawler juega antes que la víctima en el tick 0.
    let mut bw = BankWorld::from_examples("Arena", &[("Brawler", 2, 2), ("TalkerVictim", 3, 2)]);
    let victim_id = bw
        .creature_ids
        .iter()
        .find(|(name, _)| name == "TalkerVictim")
        .map(|(_, id)| *id)
        .expect("TalkerVictim colocada");
    let events = bw.step_tick();

    // La víctima murió por daño del atacante en el tick 0.
    assert!(
        events.iter().any(|e| matches!(
            e,
            instinct_core::event::Event::Died { species, pos, cause, .. }
                if species == "TalkerVictim" && *pos == instinct_core::event::Pos { x: 3, y: 2 } && cause == "health"
        )),
        "esperaba Died de TalkerVictim en (3,2), eventos: {:?}",
        events
    );

    // La víctima no emitió ningún evento de acción (ni antes ni después de
    // morir en ese tick): no Said, no Moved, no Dañado, no Roared.
    let victim_events: Vec<_> = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                instinct_core::event::Event::Said { species, pos, .. }
                    | instinct_core::event::Event::Damaged { species, pos, .. }
                    | instinct_core::event::Event::Roared { species, pos, .. }
                    if species == "TalkerVictim" && *pos == instinct_core::event::Pos { x: 3, y: 2 }
            )
        })
        .collect();
    assert!(
        victim_events.is_empty(),
        "TalkerVictim no debió jugar antes de morir, eventos de la víctima: {:?}",
        victim_events
    );

    // La víctima quedó fuera del mundo.
    assert!(
        bw.sim.world.creatures[victim_id.0].is_none(),
        "TalkerVictim slot vacía tras morir"
    );
}
