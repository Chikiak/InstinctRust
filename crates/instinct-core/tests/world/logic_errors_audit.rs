//! Regresión (auditoría R2-03): cada hallazgo A-E bloquea el comportamiento verificado.
//! No cambia código fuente: el núcleo ya es correcto; los tests impiden regresiones.

use super::common::*;

#[test]
fn test_a_creature_snapshot_returns_real_values_not_zero() {
    let mut world = empty_world(); // 3x3
    let species = Rc::new(Species {
        name: "Dwarf".into(),
        faction: "Test".into(),
        health: 12,
        vision: 5,
        lifespan: 100,
        program: None,
    });
    let id = world.place_creature(&species, 2, 1).expect("place");
    let snap = world.snapshot();
    let c = snap
        .creatures
        .iter()
        .find(|cr| cr.id == id)
        .expect("creature in snapshot");
    assert_eq!(c.health, 12, "health real, no 0 silencioso");
    assert_eq!(c.age, 0);
    assert_eq!(c.lifespan, 100);
    assert_eq!(c.x, 2);
    assert_eq!(c.y, 1);
    assert_eq!(c.species_name, "Dwarf");
}

#[test]
fn test_c_move_creature_to_out_of_bounds_no_panic() {
    let mut world = empty_world(); // 3x3
    let species = Rc::new(Species::new("Walker"));
    let id = world.place_creature(&species, 1, 1).expect("place");
    let before = world.creature_position(id).unwrap();
    let err = world.move_creature_to(id, 3, 1).unwrap_err(); // x=3 == columnas (fuera de rango)
    assert!(
        err.message.contains("outside") || err.message.contains("fuera"),
        "error de destino fuera de rango: {err:?}"
    );
    assert_eq!(
        world.creature_position(id).unwrap(),
        before,
        "la criatura no se movió"
    );
}

#[test]
fn test_d_end_of_tick_age_overflow_no_panic() {
    let mut world = empty_world();
    let species = Rc::new(Species {
        name: "Old".into(),
        faction: "Test".into(),
        health: 10,
        vision: 1,
        lifespan: i64::MAX,
        program: None,
    });
    let id = world.place_creature(&species, 0, 0).expect("place");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.age = i64::MAX;
        }
    }
    let events = world.end_of_tick_creatures();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::Died { cause, .. } if cause == "error")),
        "edad i64::MAX debe morir por error de desbordamiento, got {events:?}"
    );
    assert!(world.creatures[id.0].is_none(), "criatura eliminada");
}

#[test]
fn test_e_diagnostics_public_and_snapshot_separate() {
    let world = empty_world();
    assert!(
        world.diagnostics.is_empty(),
        "diagnostics vacío tras carga limpia: {:?}",
        world.diagnostics
    );
    let snap = world.snapshot();
    assert_eq!(snap.ticks, 0);
    assert!(snap.creatures.is_empty());
}
