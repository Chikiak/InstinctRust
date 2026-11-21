use super::BankWorld;

// AC7 (núcleo): waiter ages while sleeping.
#[test]
fn waiter_ages_while_sleeping() {
    // Napper: wait(3) → sleep_turns=2, lifespan 5.
    // Tick 0: wait(3) executes, age 0→1. Sleeping(2).
    // Tick 1: Sleeping(2)→Sleeping(1), no action, age 1→2.
    // Tick 2: Sleeping(1)→Awake, no action, age 2→3.
    // Tick 3: Awake, says "despierto", age 3→4.
    let mut bw = BankWorld::from_examples("Arena", &[("Napper", 2, 2)]);
    bw.step_ticks(3);
    let m = bw.memento("Napper").expect("Napper alive");
    assert_eq!(m.age, 3, "Napper aged 3 ticks while sleeping");
    assert_eq!(m.x, 2);
    assert_eq!(m.y, 2);
}

// AC2 (núcleo): Sprinter position after 1 tick.
#[test]
fn sprinter_position_after_tick() {
    let mut bw = BankWorld::from_examples("Arena", &[("Sprinter", 2, 2)]);
    bw.step_tick();
    let m = bw.memento("Sprinter").expect("Sprinter alive");
    assert_eq!(m.x, 5, "Sprinter x after 1 tick");
    assert_eq!(m.y, 2, "Sprinter y after 1 tick");
}

// AC6 (núcleo): Diagonal position after 1 tick.
#[test]
fn diagonal_position_after_tick() {
    let mut bw = BankWorld::from_examples("Arena", &[("Diagonal", 2, 2)]);
    bw.step_tick();
    let m = bw.memento("Diagonal").expect("Diagonal alive");
    assert_eq!(m.x, 3, "Diagonal x after 1 tick");
    assert_eq!(m.y, 3, "Diagonal y after 1 tick");
}

// AC8 (núcleo): Pacer moves on tick 0 and says "paso" on the next tick.
// Una acción por turno aunque el movimiento sea de varias casillas.
#[test]
fn pacer_moves_then_says_next_tick() {
    let mut bw = BankWorld::from_examples("Arena", &[("Pacer", 2, 2)]);
    // Tick 0 → 1: move(1,0,2) consumes the turn. Pacer at (4,2).
    let events1 = bw.step_tick();
    // El dicho no debe emitirse en el mismo tick del move (una acción por turno).
    assert!(
        !events1
            .iter()
            .any(|e| matches!(e, instinct_core::event::Event::Said { .. })),
        "tick 0: Pacer must not say anything on the move tick"
    );
    let m1 = bw.memento("Pacer").expect("Pacer alive");
    assert_eq!(m1.x, 4, "Pacer x after move tick");
    assert_eq!(m1.y, 2, "Pacer y after move tick");

    // Tick 1 → 2: say("paso") executes (free action), then goto start,
    // then move(1,0,2) consumes the turn. Pacer moves again.
    let events2 = bw.step_tick();
    let m2 = bw.memento("Pacer").expect("Pacer alive");
    assert_eq!(m2.x, 6, "Pacer x after say+move tick");
    assert_eq!(m2.y, 2, "Pacer y after say+move tick");
    assert!(
        events2.iter().any(|e| {
            matches!(e, instinct_core::event::Event::Said { text, .. } if text == "paso")
        }),
        "Pacer said 'paso' on a tick after the move"
    );
}
