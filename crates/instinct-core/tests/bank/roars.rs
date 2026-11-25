//! Pruebas de escenario F6-06 (núcleo): `roar` no se auto-oye y no cruza la
//! visión (AC3, AC4).
//!
//! Usa `BankWorld::from_examples` + `step_tick` + `eval` sobre el mapa
//! `Arena` (9x9, todo hierba). El rugido emitido en un tick solo se percibe
//! en el siguiente y solo de otras criaturas dentro del radio de visión.

use super::BankWorld;
use instinct_core::value::Value;

fn shouter_id(bw: &BankWorld) -> instinct_core::world::CreatureId {
    bw.creature_ids
        .iter()
        .find(|(n, _)| n == "Shouter")
        .map(|(_, id)| *id)
        .expect("Shouter")
}

fn listener_id(bw: &BankWorld) -> instinct_core::world::CreatureId {
    bw.creature_ids
        .iter()
        .find(|(n, _)| n == "Listener")
        .map(|(_, id)| *id)
        .expect("Listener")
}

fn unwrap_int(v: Value) -> i64 {
    match v {
        Value::Int(n) => n,
        _ => panic!("esperado int, got {v:?}"),
    }
}

fn unwrap_text(v: Value) -> String {
    match v {
        Value::Text(t) => t,
        _ => panic!("esperado texto, got {v:?}"),
    }
}

// AC3: una criatura no oye su propio rugido (roars_near cuenta a los demás).
#[test]
fn own_roar_is_not_heard() {
    let mut bw = BankWorld::from_examples("Arena", &[("Shouter", 2, 2)]);
    bw.step_tick();
    let id = shouter_id(&bw);
    assert_eq!(
        unwrap_int(bw.eval(id, "roars_near").unwrap()),
        0,
        "AC3: Shouter no oye su propio rugido"
    );
    assert_eq!(
        unwrap_text(bw.eval(id, "last_roar").unwrap()),
        "",
        "AC3: Shouter no tiene ultimo rugido ajeno"
    );
}

// AC4: un rugido fuera de la visión no se oye. (2,2) a (8,8): distancia 6 > visión 5.
#[test]
fn roar_outside_vision_is_not_heard() {
    let mut bw = BankWorld::from_examples("Arena", &[("Shouter", 2, 2), ("Listener", 8, 8)]);
    bw.step_tick();
    let id = listener_id(&bw);
    assert_eq!(
        unwrap_int(bw.eval(id, "roars_near").unwrap()),
        0,
        "AC4: Listener no oye rugido fuera de vision"
    );
    assert_eq!(
        unwrap_text(bw.eval(id, "last_roar").unwrap()),
        "",
        "AC4: Listener no tiene ultimo rugido lejano"
    );
}

// Control positivo de la rotación de rugidos (AC3/AC4): los rugidos del tick 0
// quedan legibles en `roars_near`/`last_roar` tras un solo `step_tick()`, igual
// que en `own_roar_is_not_heard` y `roar_outside_vision_is_not_heard`.
// `sim.rs` rota al final de cada tick (`rotate_roars()`), por lo que el rugido
// emitido por Shouter en el tick 0 está en `roars_previous` cuando Listener
// evalúa tras ese mismo `step_tick()`: no se evalúa antes de rotar el buffer.
#[test]
fn roar_tick_0_is_readable_next_tick() {
    let mut bw = BankWorld::from_examples("Arena", &[("Shouter", 2, 2), ("Listener", 4, 2)]);
    bw.step_tick();
    let id = listener_id(&bw);
    assert_eq!(
        unwrap_int(bw.eval(id, "roars_near").unwrap()),
        1,
        "control positivo: Listener@4,2 oye el rugido de Shouter@2,2 (dist 2 <= vision 5)"
    );
    assert_eq!(
        unwrap_text(bw.eval(id, "last_roar").unwrap()),
        "¡auau!",
        "control positivo: Listener lee el texto del rugido emitido en el tick 0"
    );
}
