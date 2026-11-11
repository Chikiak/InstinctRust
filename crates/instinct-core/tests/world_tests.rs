//! Unit tests for the `world` module.

use instinct_core::actions::ActionResult;
use instinct_core::event::Event;
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::{CellContent, CreatureId, CreatureMemento, CreatureState, World};
use std::collections::HashMap;
use std::rc::Rc;

fn vision_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
            vec!['.'; 5],
        ],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![];
    World::from_map(&map, &terrains, &objects)
}

/// Construye una especie con la visión indicada, lista para compartir por `Rc`.
/// El estado extrínseco (salud, edad, posición...) sigue en la criatura; la
/// visión es dato intrínseco de la especie y no se muta tras compartirla.
fn species_with_vision(name: &str, vision: i64) -> Rc<Species> {
    Rc::new(Species {
        name: name.to_string(),
        faction: String::new(),
        health: 0,
        vision,
        lifespan: 0,
        program: None,
    })
}

fn terrain_grass() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

fn object_rock() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

fn empty_world() -> World {
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 3], vec!['.'; 3], vec!['.'; 3]],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![object_rock()];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

#[test]
fn test_draw_example_map() {
    let map = Map {
        name: "Plain with rocks".to_string(),
        rows: vec![
            "....................".chars().collect(),
            "...##...............".chars().collect(),
        ],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![object_rock()];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    let drawing = world.draw().unwrap();
    let lines: Vec<&str> = drawing.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "....................");
    assert_eq!(lines[1], "...##...............");
}

#[test]
fn test_unknown_character_generates_diagnostic() {
    let map = Map {
        name: "Rare".to_string(),
        rows: vec!["....x....".chars().collect()],
    };
    let terrains = vec![terrain_grass()];
    let objects = vec![object_rock()];
    let world = World::from_map(&map, &terrains, &objects);
    assert!(
        world
            .diagnostics
            .iter()
            .any(|d| d.line == 2 && d.message.contains('x')),
        "{:?}",
        world.diagnostics
    );
}

// ---------------------------------------------------------------------------
// AC1: place_creature validation
// ---------------------------------------------------------------------------

#[test]
fn test_place_rejects_invalid_cells() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));

    // Outside the map.
    let err = world.place_creature(&species, 5, 5).unwrap_err();
    assert!(
        err.message.contains("(5,5)"),
        "error names coordinates: {err:?}"
    );
    assert!(
        world.creatures.is_empty(),
        "world unchanged on failed place"
    );

    // On an object: rock at (0,0) in empty_world? No, objects list is separate.
    // Place a rock object on the grid manually: cell (0,0) is grass only.
    // Instead, place a creature first, then try to place another on top.
    let id1 = world
        .place_creature(&species, 1, 1)
        .expect("place first creature");
    assert_eq!(world.creatures.len(), 1);
    let err = world.place_creature(&species, 1, 1).unwrap_err();
    assert!(
        err.message.contains("(1,1)"),
        "error names coordinates: {err:?}"
    );
    assert_eq!(world.creatures.len(), 1, "second creature not placed");
    // First creature still there.
    assert_eq!(
        world.creature_position(id1).unwrap(),
        instinct_core::event::Pos { x: 1, y: 1 }
    );
}

#[test]
fn test_place_creature_returns_stable_id_and_reuses_slot() {
    let mut world = empty_world();
    let id1 = world
        .place_creature(&Rc::new(Species::new("A")), 0, 0)
        .expect("place A");
    let id2 = world
        .place_creature(&Rc::new(Species::new("B")), 2, 2)
        .expect("place B");
    assert_eq!(id1, CreatureId(0));
    assert_eq!(id2, CreatureId(1));
    assert_ne!(id1, id2);
    // Las criaturas se consultan O(1) por id estable.
    assert_eq!(
        world.creature_position(id1).unwrap(),
        instinct_core::event::Pos { x: 0, y: 0 }
    );
    assert_eq!(
        world.creature_position(id2).unwrap(),
        instinct_core::event::Pos { x: 2, y: 2 }
    );

    // Remover id1 libera su hueco sin tocar a id2.
    world.remove_creature(id1).expect("remove A");
    assert!(world.creature_position(id1).is_err());
    assert_eq!(
        world.creature_position(id2).unwrap(),
        instinct_core::event::Pos { x: 2, y: 2 }
    );
    assert_eq!(
        world
            .creatures
            .iter()
            .enumerate()
            .filter_map(|(i, s)| s.as_ref().map(|_| i))
            .collect::<Vec<_>>(),
        vec![1]
    );

    // Un place posterior reusa ese mismo id (0).
    let id3 = world
        .place_creature(&Rc::new(Species::new("C")), 0, 0)
        .expect("place C");
    assert_eq!(id3, id1, "el hueco libre se reusa con el mismo id");
    assert_ne!(id3, id2);
    assert_eq!(
        world.creature_position(id3).unwrap(),
        instinct_core::event::Pos { x: 0, y: 0 }
    );
    // La otra criatura no se corrompe.
    assert_eq!(
        world.creature_position(id2).unwrap(),
        instinct_core::event::Pos { x: 2, y: 2 }
    );
    assert_eq!(
        world.creatures.iter().filter(|c| c.is_some()).count(),
        2,
        "dos criaturas vivas tras reusar el hueco"
    );
}

// ---------------------------------------------------------------------------
// AC2: render shows first letter of species
// ---------------------------------------------------------------------------

#[test]
fn test_render_shows_creature() {
    let mut world = empty_world();
    world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place Walker");
    let drawing = world.draw().unwrap();
    let lines: Vec<&str> = drawing.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "...");
    assert_eq!(lines[1], ".W.");
    assert_eq!(lines[2], "...");

    // Object still draws its character.
    let mut world2 = empty_world();
    // El catálogo del mundo tiene roca, pero la rejilla no tiene objetos:
    // solo confirma que la criatura dibuja sobre el terreno.
    world2
        .place_creature(&Rc::new(Species::new("Ent")), 0, 0)
        .expect("place Ent");
    let drawing2 = world2.draw().unwrap();
    let lines2: Vec<&str> = drawing2.lines().collect();
    assert_eq!(lines2[0], "E..");
}

#[test]
fn test_remove_creature_frees_cell() {
    let mut world = empty_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place");
    assert!(matches!(
        world.content[1][1],
        CellContent::Creature(cid) if cid == id
    ));
    world.remove_creature(id).expect("remove");
    assert!(matches!(world.content[1][1], CellContent::Empty));
    assert!(world.creatures[id.0].is_none(), "el hueco quedó libre");
    assert!(
        world.creatures.iter().all(|c| c.is_none()),
        "el almacén solo tiene huecos libres"
    );
}

// ---------------------------------------------------------------------------
// W6: robustez de entradas y aritmética comprobada en fin de tick
// ---------------------------------------------------------------------------

#[test]
fn test_end_of_tick_min_health_no_panic() {
    // Vida en i64::MIN: health - 1 desborda; no debe pánico, la criatura muere.
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world.place_creature(&species, 0, 0).expect("place");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = i64::MIN;
            c.age = 0;
            c.lifespan = i64::MAX;
        }
    }
    let events = world.end_of_tick_creatures();
    assert!(
        events.iter().any(|e| matches!(e, Event::Died { .. })),
        "la criatura con vida i64::MIN debe morir, got {events:?}"
    );
    assert!(
        world.creatures[id.0].is_none(),
        "criatura removida del mapa"
    );
    assert!(
        world.creatures.iter().all(|c| c.is_none()),
        "el almacén solo tiene huecos libres"
    );
}

#[test]
fn test_damage_cell_overflow_and_out_of_map() {
    let mut world = empty_world();
    // Terreno (0,0) con reserve 100. Daño i64::MIN -> 100 - i64::MIN desborda.
    let err = world.damage_cell_content(0, 0, i64::MIN).unwrap_err();
    assert!(
        err.message.contains("overflow"),
        "debe reportar overflow, got {err:?}"
    );
    // La reserva no cambió ante el overflow.
    assert_eq!(world.terrains[0].reserve, 100);

    // Fuera del mapa: no-op, sin error.
    let ok = world.damage_cell_content(999, 999, 100);
    assert!(ok.is_ok(), "fuera del mapa es no-op");
    assert_eq!(world.terrains[0].reserve, 100);
}

#[test]
fn test_move_creature_invalid_inputs_error() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world.place_creature(&species, 1, 1).expect("place");

    // Identificador inexistente.
    let err = world.move_creature_to(CreatureId(9999), 1, 1).unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "error por id desconocido, got {err:?}"
    );

    // Destino fuera de rango: error controlado, sin pánico.
    let err = world.move_creature_to(id, 999, 999).unwrap_err();
    assert!(
        err.message.contains("outside") || err.message.contains("fuera"),
        "error por destino fuera de rango, got {err:?}"
    );
    // La criatura no se movió.
    assert_eq!(
        world.creature_position(id).unwrap(),
        instinct_core::event::Pos { x: 1, y: 1 }
    );
}

#[test]
fn test_world_setters_unknown_id_error() {
    let mut world = empty_world();
    let unknown = CreatureId(9999);

    let err = world
        .apply_action_result(
            unknown,
            ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            },
        )
        .unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "apply_action_result id desconocido, got {err:?}"
    );

    let err = world.set_creature_health(unknown, 10).unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "set_creature_health id desconocido, got {err:?}"
    );

    let err = world
        .set_variable(unknown, "tesoro", Value::Int(1))
        .unwrap_err();
    assert!(
        err.message.contains("unknown creature"),
        "set_variable id desconocido, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// AC3: draw devuelve error ante identificador de criatura huérfano.
// ---------------------------------------------------------------------------

#[test]
fn test_draw_no_orphan_panic() {
    let mut world = empty_world();
    let id = world
        .place_creature(&Rc::new(Species::new("Walker")), 1, 1)
        .expect("place Walker");
    // Huérfano: la celda apunta a un id cuyo hueco ya se liberó; draw debe
    // reportarlo como error, nunca pánico.
    world.content[1][1] = CellContent::Creature(id);
    world.creatures[id.0] = None;

    let result = world.draw();
    assert!(
        result.is_err(),
        "draw debe fallar con criatura huérfana, no pánico"
    );
    let err = result.unwrap_err();
    assert_eq!(err.file, "world");
    assert!(
        err.message.contains(&id.0.to_string()),
        "error nombra el id huérfano: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// AC4: draw no panica cuando el catálogo no cubre el índice de la rejilla.
// ---------------------------------------------------------------------------

#[test]
fn test_draw_short_catalog_errors() {
    let mut world = empty_world();
    // content[0][0] referencia al objeto 5, pero el catálogo solo tiene 1 objeto.
    world.content[0][0] = CellContent::Object(5);

    let result = world.draw();
    assert!(
        result.is_err(),
        "draw debe fallar con catálogo de objetos corto"
    );
    let err = result.unwrap_err();
    assert!(err.message.contains("5"), "error nombra el índice: {err:?}");

    // Catálogo de terrenos corto: el terreno de la celda no existe (el mundo tiene 1 terreno).
    let mut world2 = empty_world();
    world2.terrain[0][0] = 9;
    let result2 = world2.draw();
    assert!(
        result2.is_err(),
        "draw debe fallar con catálogo de terrenos corto"
    );
    let err2 = result2.unwrap_err();
    assert!(
        err2.message.contains("9"),
        "error nombra el índice: {err2:?}"
    );
}

// ---------------------------------------------------------------------------
// AC2: vision radius is Chebyshev
// ---------------------------------------------------------------------------

#[test]
fn test_vision_radius_is_chebyshev() {
    // AC2: Eight neighbor cells at distance 1; cell at distance exactly vision
    // enters; cell at vision + 1 stays out.
    let mut world = vision_world(); // 5x5, all grass
    let id = world
        .place_creature(&species_with_vision("Walker", 1), 2, 2)
        .expect("place center");

    // All 8 neighbors at Chebyshev distance 1 should be within vision 1.
    for (x, y) in [
        (1, 1),
        (1, 2),
        (1, 3),
        (2, 1),
        (2, 3),
        (3, 1),
        (3, 2),
        (3, 3),
    ] {
        world
            .place_creature(&Rc::new(Species::new("Ally")), x, y)
            .expect("place neighbor");
    }
    let nb = world.nearby(id);
    assert_eq!(nb.allies, 8, "all 8 neighbors are within vision 1");

    // Cells at distance 2 should be out of vision 1: (0, 2) is at distance 2.
    let _id_out = world
        .place_creature(&Rc::new(Species::new("Far")), 0, 2)
        .expect("place at distance 2");
    let nb = world.nearby(id);
    assert_eq!(nb.allies, 8, "distance 2 (vision + 1) is out of vision 1");

    // With vision 2, the cell at distance 2 enters.
    let mut world2 = vision_world();
    let id2 = world2
        .place_creature(&species_with_vision("Walker", 2), 2, 2)
        .expect("place center");
    let _id_at_dist2 = world2
        .place_creature(&Rc::new(Species::new("Ally")), 0, 2)
        .expect("place at distance 2");
    let nb2 = world2.nearby(id2);
    assert_eq!(nb2.allies, 1, "cell at distance exactly vision enters");
    assert_eq!(nb2.ally.unwrap().dist, 2);
}

// ---------------------------------------------------------------------------
// AC4: nearest tie-break is stable
// ---------------------------------------------------------------------------

#[test]
fn test_nearest_tie_break_is_stable() {
    let mut world = vision_world(); // 5x5
    let id = world
        .place_creature(&species_with_vision("Walker", 2), 2, 2)
        .expect("place center");

    // Two allies at the same distance (2): (0, 2) has lower id than (4, 2)
    // since it's placed first. Both at Chebyshev distance 2.
    let _id_a1 = world
        .place_creature(&Rc::new(Species::new("Ally1")), 0, 2)
        .expect("place ally at (0,2)");
    let _id_a2 = world
        .place_creature(&Rc::new(Species::new("Ally2")), 4, 2)
        .expect("place ally at (4,2)");

    let nb = world.nearby(id);
    assert_eq!(nb.allies, 2);
    // First placed (id ascending) wins the tie at same distance.
    assert_eq!(nb.ally.unwrap().dist, 2);
    assert_eq!(nb.ally.unwrap().dx, -1, "dx toward (0,2) from (2,2)");
    assert_eq!(nb.ally.unwrap().dy, 0);

    // Verify stability: reading again gives the same result.
    let nb2 = world.nearby(id);
    assert_eq!(nb2.ally, nb.ally);
    assert_eq!(nb2.allies, nb.allies);
}

// ---------------------------------------------------------------------------
// Memento: captura y restauración de estado de criatura para el turno
// ---------------------------------------------------------------------------

#[test]
fn test_creature_memento_captures_state() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world
        .place_creature(&species, 2, 2)
        .expect("place creature");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = 42;
            c.age = 7;
            c.pc = 5;
            c.state = CreatureState::Sleeping(3);
            c.variables.insert("oro".to_string(), Value::Int(100));
            c.variables.insert("vida_extra".to_string(), Value::Int(10));
        }
    }

    let memento = world.creatures[id.0].as_ref().expect("creature").memento();

    assert_eq!(memento.id, id);
    assert_eq!(memento.health, 42);
    assert_eq!(memento.age, 7);
    assert_eq!(memento.x, 2);
    assert_eq!(memento.y, 2);
    assert_eq!(memento.pc, 5);
    assert_eq!(memento.state, CreatureState::Sleeping(3));
    assert_eq!(memento.variables.get("oro"), Some(&Value::Int(100)));
    assert_eq!(memento.variables.get("vida_extra"), Some(&Value::Int(10)));
}

#[test]
fn test_restore_creature_from_memento() {
    let mut world = empty_world();
    let species = Rc::new(Species::new("Walker"));
    let id = world
        .place_creature(&species, 1, 1)
        .expect("place creature");
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = 10;
            c.age = 0;
            c.pc = 0;
            c.state = CreatureState::Awake;
        }
    }
    let memento = world.creatures[id.0].as_ref().expect("creature").memento();

    // Mutar el estado de la criatura.
    for c in world.creatures.iter_mut().flatten() {
        if c.id == id {
            c.health = 1;
            c.age = 99;
            c.x = 9;
            c.y = 9;
            c.pc = 99;
            c.state = CreatureState::Sleeping(5);
            c.variables.insert("basura".to_string(), Value::Int(-1));
        }
    }

    let restored = world.restore_creature_from_memento(&memento);
    assert!(restored, "la restauración debe tener éxito");

    let c = world.creatures[id.0].as_ref().expect("creature");
    assert_eq!(c.health, 10);
    assert_eq!(c.age, 0);
    assert_eq!((c.x, c.y), (1, 1));
    assert_eq!(c.pc, 0);
    assert_eq!(c.state, CreatureState::Awake);
    assert!(
        c.variables.is_empty(),
        "las variables deben restaurarse vacías"
    );
}

#[test]
fn test_restore_creature_from_memento_unknown_id() {
    let mut world = empty_world();
    let memento = CreatureMemento {
        id: CreatureId(9999),
        health: 0,
        age: 0,
        x: 0,
        y: 0,
        pc: 0,
        state: CreatureState::Awake,
        variables: HashMap::new(),
    };
    assert!(
        !world.restore_creature_from_memento(&memento),
        "restaurar id desconocido debe fallar sin pánico"
    );
}

// ---------------------------------------------------------------------------
// AC1: dos criaturas de la misma especie comparten el mismo Rc<Species>
// ---------------------------------------------------------------------------

#[test]
fn test_creatures_share_species_rc() {
    let mut world = empty_world();
    let grass = Rc::new(Species::new("Grass"));

    let id1 = world
        .place_creature(&grass, 0, 0)
        .expect("place primera criatura");
    let id2 = world
        .place_creature(&grass, 1, 1)
        .expect("place segunda criatura");

    let c1 = world.creatures[id1.0].as_ref().expect("criatura 1 viva");
    let c2 = world.creatures[id2.0].as_ref().expect("criatura 2 viva");

    // Ambas criaturas apuntan a la misma instancia de Species.
    assert!(
        Rc::ptr_eq(&c1.species, &c2.species),
        "criaturas de la misma especie deben compartir el Rc<Species>"
    );
    // Y el mundo registró su propia copia en el catálogo con el mismo puntero.
    assert!(
        world
            .species
            .iter()
            .any(|owned| Rc::ptr_eq(owned, &c1.species)),
        "el catálogo del mundo es dueño del Rc<Species> compartido"
    );
    // draw sigue mostrando la primera letra de la especie compartida.
    let drawing = world.draw().unwrap();
    let lines: Vec<&str> = drawing.lines().collect();
    assert_eq!(lines[0], "G..");
    assert_eq!(lines[1], ".G.");
    assert_eq!(lines[2], "...");
}

// ---------------------------------------------------------------------------
// AC2 / W2: un único dueño consistente de las reservas de terreno y objeto.
// World y Simulation nunca mantienen copias desincronizadas: el daño y el
// fin de tick actúan sobre el mismo catálogo.
// ---------------------------------------------------------------------------

#[test]
fn test_terrain_reserve_consistent_world_simulation() {
    let mut world = empty_world();
    // (2,2) es hierba del catálogo único del mundo: reserve 100.
    assert_eq!(world.terrains[0].reserve, 100);
    assert_eq!(world.reserve_at(2, 2), Some(100));

    // Daño sobre el catálogo del mundo (lo mismo que muta strike_cell).
    world
        .damage_cell_content(2, 2, 30)
        .expect("daña terreno de (2,2)");
    assert_eq!(world.terrains[0].reserve, 70);
    assert_eq!(world.reserve_at(2, 2), Some(70));

    // La simulación lee y regenera sobre el mismo catálogo: no tiene copia propia.
    let mut sim = Simulation::new(world);
    let _ = sim.step_tick(); // fin de tick: regenera +2 sobre la reserva dañada -> 72

    // World.reserve_at y la reserva de Simulation coinciden: un único dueño.
    assert_eq!(sim.world().reserve_at(2, 2), Some(72));
    assert_eq!(sim.world.terrains[0].reserve, 72);
    assert_eq!(sim.world.tick(), 1);
}

#[test]
fn test_object_reserve_consistent_world_simulation() {
    let mut world = empty_world();
    // Roca del catálogo en (0,0): reserve 100.
    world.content[0][0] = CellContent::Object(0);
    assert_eq!(world.objects[0].reserve, 100);
    assert_eq!(world.object_name_at(0, 0), Some("Rock"));

    // Agota el objeto sobre el catálogo del mundo.
    world
        .damage_cell_content(0, 0, 100)
        .expect("agota objeto de (0,0)");
    assert_eq!(world.objects[0].reserve, 0);

    // Fin de tick en la simulación sobre el mismo catálogo: el objeto se retira.
    let mut sim = Simulation::new(world);
    let _ = sim.step_tick();

    assert!(
        matches!(sim.world.content[0][0], CellContent::Empty),
        "el objeto agotado libera su casilla"
    );
    assert_eq!(sim.world.objects[0].reserve, 0);
    assert_eq!(sim.world.object_name_at(0, 0), None);
}
