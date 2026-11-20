//! Headless tests for `roar_markers` (pure render, no window, no eframe).
//!
//! Pattern: `tests/render/roars.rs` — unit tests + integration test against
//! `examples/` with a roaring creature.

use instinct_core::event::{Event, Pos};
use instinct_core::world::{CreatureId, Roar};
use instinct_gui::render::roars::roar_markers;

/// t-roar-01: un rugido de este tick marca la celda y el texto.
#[test]
fn roar_markers_este_tick_marca_celda_y_texto() {
    let roars = vec![Roar {
        tick: 5,
        creature_id: CreatureId(0),
        species: "Ent".to_string(),
        pos: Pos { x: 3, y: 2 },
        text: "hoom".to_string(),
    }];
    let markers = roar_markers(&roars, 12, 16);
    assert!(markers.contains(&(3, 2, "hoom".to_string())));
}

/// t-roar-02: un rugido del tick anterior también marca la celda.
#[test]
fn roar_markers_tick_anterior_marca_celda() {
    let roars = vec![Roar {
        tick: 4,
        creature_id: CreatureId(0),
        species: "Dwarf".to_string(),
        pos: Pos { x: 0, y: 0 },
        text: "khazad".to_string(),
    }];
    let markers = roar_markers(&roars, 12, 16);
    assert!(markers.contains(&(0, 0, "khazad".to_string())));
}

/// t-roar-03: lista vacía no produce marcadores.
#[test]
fn roar_markers_vacio_sin_marcadores() {
    let markers = roar_markers(&[], 12, 16);
    assert!(markers.is_empty());
}

/// t-roar-04: varios rugidos en celdas distintas producen ambas entradas.
#[test]
fn roar_markers_multiple_roars() {
    let roars = vec![
        Roar {
            tick: 5,
            creature_id: CreatureId(0),
            species: "Ent".to_string(),
            pos: Pos { x: 3, y: 2 },
            text: "hoom".to_string(),
        },
        Roar {
            tick: 5,
            creature_id: CreatureId(1),
            species: "Dwarf".to_string(),
            pos: Pos { x: 7, y: 9 },
            text: "khazad".to_string(),
        },
    ];
    let markers = roar_markers(&roars, 12, 16);
    assert!(markers.contains(&(3, 2, "hoom".to_string())));
    assert!(markers.contains(&(7, 9, "khazad".to_string())));
    assert_eq!(markers.len(), 2);
}

/// t-roar-05: cada entrada tiene texto no vacío.
#[test]
fn roar_markers_texto_no_vacio() {
    let roars = vec![
        Roar {
            tick: 5,
            creature_id: CreatureId(0),
            species: "Ent".to_string(),
            pos: Pos { x: 3, y: 2 },
            text: "hoom".to_string(),
        },
        Roar {
            tick: 4,
            creature_id: CreatureId(1),
            species: "Dwarf".to_string(),
            pos: Pos { x: 0, y: 0 },
            text: "khazad".to_string(),
        },
    ];
    let markers = roar_markers(&roars, 12, 16);
    for (_, _, text) in &markers {
        assert!(!text.is_empty(), "texto vacío en marcador: {text:?}");
    }
}

// ---------------------------------------------------------------------------
// Integración: sesión con examples/, criatura que ruge, start_simulation,
// step_tick, snapshot(), eventos, previous_roars().
// ---------------------------------------------------------------------------

use instinct_core::event::WorldOutput;
use instinct_core::sim::Simulation;
use instinct_core::world::World;
use std::cell::RefCell;
use std::rc::Rc;

/// Puerto auxiliar para conectar el mundo a un LogBuffer en tests.
fn test_port(buffer: Rc<RefCell<instinct_gui::app::log::LogBuffer>>) -> Box<dyn WorldOutput> {
    struct LocalPort(Rc<RefCell<instinct_gui::app::log::LogBuffer>>);
    impl WorldOutput for LocalPort {
        fn emit(&mut self, event: &instinct_core::event::Event) {
            self.0.borrow_mut().push_event(event);
        }
    }
    Box::new(LocalPort(buffer))
}

/// Construye una simulación desde examples con un Roarer en (1, 0) y seed 7.
fn build_roarer_sim() -> (Simulation, Rc<RefCell<instinct_gui::app::log::LogBuffer>>) {
    let session = instinct_gui::app::session::LoadedSession::new(&examples_root());
    let map = session
        .maps()
        .iter()
        .find(|m| m.name == "Plain with rocks")
        .expect("map 'Plain with rocks'");
    let species = session
        .species()
        .iter()
        .find(|s| s.name == "Roarer")
        .expect("species 'Roarer'");

    let mut world = World::from_map(
        map,
        &session.resources().terrains,
        &session.resources().objects,
    );
    let species_rc = Rc::new(species.clone());
    world
        .place_creature(&species_rc, 1, 0)
        .expect("place Roarer ok");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);
    let buffer = Rc::new(RefCell::new(
        instinct_gui::app::log::LogBuffer::with_capacity(1000),
    ));
    sim.world.set_output(test_port(Rc::clone(&buffer)));
    (sim, buffer)
}

fn examples_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// t-roar-06: tras step_tick, los eventos devueltos contienen Event::Roared
/// con pos de la criatura y texto no vacío; roar_markers devuelve entrada
/// en esa celda.
#[test]
fn roar_integracion_eventos_tiene_roar_este_tick() {
    let (mut sim, _buffer) = build_roarer_sim();

    // Tick 0 -> step_tick avanza a tick 1 y el Roarer ruge en el tick 1.
    let events = sim.step_tick();
    let roar_events: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Roared { .. }))
        .collect();
    assert!(
        !roar_events.is_empty(),
        "debe haber al menos un Event::Roared"
    );

    let snapshot = sim.world().snapshot();
    assert_eq!(snapshot.ticks, 1);

    // Cada Event::Roared debe tener tick == 0 (emitido durante el turno en
    // tick 0, antes de que advance_tick suba el contador a 1), pos válida
    // y texto no vacío.
    for ev in &roar_events {
        if let Event::Roared {
            tick, pos, text, ..
        } = ev
        {
            assert_eq!(*tick, 0);
            assert!(!text.is_empty());
            // Los marcadores deben incluir esa celda.
            let markers = roar_markers(
                &[Roar {
                    tick: *tick,
                    creature_id: CreatureId(0),
                    species: "Roarer".to_string(),
                    pos: *pos,
                    text: text.clone(),
                }],
                12,
                16,
            );
            assert!(
                markers.contains(&(pos.x, pos.y, text.clone())),
                "marcador faltante para rugido en ({}, {})",
                pos.x,
                pos.y
            );
        }
    }
}

/// t-roar-07: tras rotar 2 ticks, previous_roars() refleja el tick anterior.
#[test]
fn roar_integracion_previous_roars_despues_de_rotar() {
    let (mut sim, _buffer) = build_roarer_sim();

    // Avanzar 2 ticks.
    for _ in 0..2 {
        sim.step_tick();
    }

    // Después de 2 step_tick, el tick es 2.
    // previous_roars() contiene los rugidos del tick 1 (rotados al final del tick 1).
    let prev = sim.world().previous_roars();
    // Puede estar vacío si el Roarer no rugió en el tick 1, pero si hay rugidos
    // del tick 1, deben tener tick == 1.
    for roar in prev {
        assert_eq!(
            roar.tick, 1,
            "previous_roars debe tener tick 1, fue {}",
            roar.tick
        );
    }

    // snapshot.ticks == 2.
    let snapshot = sim.world().snapshot();
    assert_eq!(snapshot.ticks, 2);

    // Los rugidos del tick anterior vienen de previous_roars (tick 1).
    // Los rugidos de este tick (tick == 2) vienen de los eventos del último
    // step_tick (no de snapshot.events, que se vacía al final de cada tick).
    let mut all_roars: Vec<Roar> = prev.to_vec();

    // Volver a ejecutar un step_tick para capturar eventos "de este tick".
    let events = sim.step_tick();
    for ev in &events {
        if let Event::Roared {
            tick,
            species,
            pos,
            text,
            ..
        } = ev
        {
            all_roars.push(Roar {
                tick: *tick,
                creature_id: CreatureId(0),
                species: species.clone(),
                pos: *pos,
                text: text.clone(),
            });
        }
    }

    let markers = roar_markers(&all_roars, 12, 16);
    // Cada marcador debe tener texto no vacío y coords válidas.
    for (x, y, text) in &markers {
        assert!(*x < 16);
        assert!(*y < 12);
        assert!(!text.is_empty());
    }
}
