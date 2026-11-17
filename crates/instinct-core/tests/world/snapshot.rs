//! Read-only structured snapshot tests for `World::snapshot`.
//! TDD (R2-02): written first; they must fail to compile until
//! `world/snapshot_view.rs` exists, then fail on assertions until the
//! DTO reflects real world state, then pass.

use super::common::*;

#[test]
fn snapshot_before_after_equals() {
    // AC1: `snapshot(&self)` does not mutate `self`; two consecutive
    // snapshots are equal, ticks unchanged, events not drained.
    let mut world = empty_world();
    world.emit_event(Event::Said {
        tick: 0,
        species: "Test".into(),
        pos: Pos { x: 0, y: 0 },
        text: "hi".into(),
    });

    let snap1 = world.snapshot();
    let snap2 = world.snapshot();

    assert_eq!(snap1, snap2, "two snapshots of the same world are equal");
    assert_eq!(
        snap1.ticks, snap2.ticks,
        "ticks unchanged between snapshots"
    );
    assert_eq!(snap1.ticks, 0, "no tick advanced");
    assert_eq!(
        snap1.events, snap2.events,
        "events not drained by taking a snapshot"
    );
    assert_eq!(
        snap1.events.len(),
        1,
        "the emitted event is in the snapshot"
    );
    // The world is left untouched: the event is still in the log.
    assert_eq!(
        world.events.len(),
        1,
        "world event log not drained by snapshot"
    );
}

#[test]
fn snapshot_read_only_fields() {
    // AC2: `WorldSnapshot` exposes its data through public, read-only fields;
    // there are no `&mut self` methods on the DTO.
    let world = empty_world();
    let snap = world.snapshot();

    let _columns: usize = snap.columns;
    let _rows: usize = snap.rows;
    let _ticks: u64 = snap.ticks;
    let _map_name: &str = &snap.map_name;
    let _cells: &Vec<Vec<CellSnapshot>> = &snap.cells;
    let _creatures: &Vec<LiveCreature> = &snap.creatures;
    let _objects: &Vec<ObjectInfo> = &snap.objects;
    let _terrains: &Vec<TerrainInfo> = &snap.terrains;
    let _events: &Vec<Event> = &snap.events;

    assert_eq!(snap.columns, 3);
    assert_eq!(snap.rows, 3);
    assert_eq!(snap.ticks, 0);
    assert_eq!(snap.map_name, "test");
}

#[test]
fn snapshot_reflects_real_state() {
    // AC3: placing a creature, advancing a tick, and comparing snapshots
    // shows ticks incremented, health decremented, age incremented, and
    // events present in the pre-tick snapshot.
    let mut sim = Simulation::new(empty_world());
    let species = Rc::new(Species {
        name: "Dwarf".into(),
        faction: "Test".into(),
        health: 10,
        vision: 1,
        lifespan: 100,
        program: None,
    });
    let id = sim
        .world
        .place_creature(&species, 1, 1)
        .expect("place dwarf");

    sim.world.emit_event(Event::Said {
        tick: 0,
        species: "Dwarf".into(),
        pos: Pos { x: 1, y: 1 },
        text: "hello".into(),
    });

    let before = sim.world.snapshot();
    assert_eq!(before.ticks, 0);
    assert_eq!(before.events.len(), 1, "events present in the snapshot");
    let before_c = before
        .creatures
        .iter()
        .find(|c| c.id == id)
        .expect("creature in the pre-tick snapshot");
    assert_eq!(before_c.health, 10);
    assert_eq!(before_c.age, 0);

    let _emitted = sim.step_tick();

    let after = sim.world.snapshot();
    assert_eq!(after.ticks, 1, "tick advanced");
    let after_c = after
        .creatures
        .iter()
        .find(|c| c.id == id)
        .expect("creature still alive after the tick");
    assert_eq!(after_c.health, 9, "health decremented by end of tick");
    assert_eq!(after_c.age, 1, "age incremented by end of tick");
    assert_eq!(after.events.len(), 0, "events drained by step_tick");
}

#[test]
fn snapshot_cell_contents() {
    // `empty_world` has a 3x3 grass grid; the catalog knows a rock but no
    // cell holds an object, so every cell has `object_char == None`.
    let mut world = empty_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place walker");

    let snap = world.snapshot();
    assert_eq!(snap.cells.len(), 3);
    for (y, row) in snap.cells.iter().enumerate() {
        assert_eq!(row.len(), 3);
        for (x, cell) in row.iter().enumerate() {
            assert_eq!(cell.terrain_char, '.', "grass terrain at ({x},{y})");
            assert_eq!(cell.object_char, None, "no objects on the empty_world grid");
            if (x, y) == (1, 1) {
                assert_eq!(cell.creature, Some(id), "creature id in its cell");
            } else {
                assert_eq!(cell.creature, None, "no creature at ({x},{y})");
            }
        }
    }
}

#[test]
fn snapshot_objects_and_terrains_catalogs() {
    let world = empty_world();
    let snap = world.snapshot();

    // Objects: index/name/char/reserve from the object catalog.
    assert_eq!(snap.objects.len(), 1);
    assert_eq!(snap.objects[0].index, 0);
    assert_eq!(snap.objects[0].name, "Rock");
    assert_eq!(snap.objects[0].char, '#');
    assert_eq!(snap.objects[0].reserve, 100);

    // Terrains: name/char/reserve/resource_max from the terrain catalog.
    assert_eq!(snap.terrains.len(), 1);
    assert_eq!(snap.terrains[0].name, "Grass");
    assert_eq!(snap.terrains[0].char, '.');
    assert_eq!(snap.terrains[0].reserve, 100);
    assert_eq!(snap.terrains[0].resource_max, 100);
}

#[test]
fn snapshot_events_clone() {
    // Events clone and the original snapshot is left intact.
    let mut world = empty_world();
    world.emit_event(Event::Said {
        tick: 3,
        species: "Dwarf".into(),
        pos: Pos { x: 1, y: 1 },
        text: "hello".into(),
    });

    let snap = world.snapshot();
    let cloned = snap.events.clone();

    assert_eq!(cloned, snap.events);
    assert_eq!(cloned.len(), 1);
    assert_eq!(world.events.len(), 1, "original event log intact");
    assert_eq!(snap.events.len(), 1, "snapshot events intact");
}

#[test]
fn snapshot_clone_debug() {
    // `WorldSnapshot` is `Clone` + `Debug`; the clone equals the original.
    let world = empty_world();
    let snap = world.snapshot();

    let cloned = snap.clone();
    assert_eq!(cloned, snap, "clone equals original");

    let debug = format!("{snap:?}");
    assert!(!debug.is_empty(), "Debug produces output");
    assert!(
        debug.contains("WorldSnapshot"),
        "Debug names the type: {debug}"
    );
}
