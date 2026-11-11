//! Scenario tests for the Simulation (sim.rs).
//! Covers AC5 (death), AC6 (terrain/objects end-of-tick), AC8 (determinism).

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::{Event, Pos};
use instinct_core::perception::PerceptionCatalog;
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::rng::RngSource;
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world::{CellContent, CreatureId, CreatureState, World};
use std::cell::RefCell;
use std::rc::Rc;

/// Deterministic RNG for tests: returns values from a fixed sequence.
#[derive(Debug, Clone)]
pub struct TestRng {
    values: Vec<i64>,
    index: usize,
}

impl TestRng {
    pub fn new(values: Vec<i64>) -> Self {
        TestRng { values, index: 0 }
    }
}

impl RngSource for TestRng {
    fn next_bounded(&mut self, max: i64) -> i64 {
        let v = self.values[self.index % self.values.len()];
        self.index += 1;
        if max < 0 {
            return 0;
        }
        let range = (max as u64).wrapping_add(1);
        (v as u64 % range) as i64
    }

    fn shuffle(&mut self, list: &mut [usize]) {
        let mut i = list.len();
        while i > 1 {
            i -= 1;
            let j = self.next_bounded(i as i64) as usize;
            list.swap(i, j);
        }
    }
}

fn grass_terrain() -> Terrain {
    Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 2,
        reserve: 100,
    }
}

fn rock_object() -> Object {
    Object {
        name: "Rock".to_string(),
        character: '#',
        resource_max: 100,
        reserve: 100,
    }
}

fn empty_map() -> Map {
    Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    }
}

/// Compiles un `.ins` y devuelve la `Species` con su programa envuelta en
/// `Rc` para compartirla por el catálogo del mundo (Flyweight).
fn species_with_program(name: &str, health: i64, lifespan: i64, body: &str) -> Rc<Species> {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text = format!(
        "creature {name}\nfaction any\nhealth {health}\nvision 3\nlifespan {lifespan}\nstart:\n{body}"
    );
    let result = compile::compile(&text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("header");
    Rc::new(Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    ))
}

fn species_no_program(name: &str, health: i64, lifespan: i64) -> Rc<Species> {
    Rc::new(Species {
        name: name.to_string(),
        faction: "any".to_string(),
        health,
        vision: 3,
        lifespan,
        program: None,
    })
}

/// Compila un `.ins` con la facción indicada y devuelve la `Species` con su
/// programa envuelta en `Rc`. Sirve para probar percepciones que leen el
/// mundo (facción, totales).
fn species_with_program_and_faction(
    name: &str,
    faction: &str,
    health: i64,
    lifespan: i64,
    body: &str,
) -> Rc<Species> {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text = format!(
        "creature {name}\nfaction {faction}\nhealth {health}\nvision 3\nlifespan {lifespan}\nstart:\n{body}"
    );
    let result = compile::compile(&text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("header");
    Rc::new(Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    ))
}

fn build_world(map: &Map, terrains: &[Terrain], objects: &[Object]) -> World {
    let world = World::from_map(map, terrains, objects);
    assert!(world.diagnostics.is_empty(), "{:?}", world.diagnostics);
    world
}

// ---------------------------------------------------------------------------
// AC5: death by health and by age.
// ---------------------------------------------------------------------------

#[test]
fn test_death_by_health_and_by_age() {
    // Creature A: health 10, lifespan 100 -> dies by salud at end of tick 10.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_a = species_no_program("A", 10, 100);
    let id_a = world.place_creature(&sp_a, 0, 0).expect("place A");

    let mut sim = Simulation::new(world);

    // 9 ticks: alive, health goes 10 -> 1.
    for _ in 0..9 {
        let events = sim.step_tick();
        assert!(
            events.iter().all(|e| !matches!(e, Event::Died { .. })),
            "no death expected before tick 10, got: {events:?}"
        );
    }
    let creature = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id_a)
        .expect("creature alive after 9 ticks");
    assert_eq!(creature.health, 1);
    assert_eq!(creature.age, 9);

    // Tick 10: health 1 -> 0, dies by vida.
    let events = sim.step_tick();
    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(died.len(), 1);
    match &died[0] {
        Event::Died {
            tick,
            species,
            cause,
            ..
        } => {
            assert_eq!(*tick, 9); // tick counter is 9 before incrementing to 10
            assert_eq!(species, "A");
            assert_eq!(cause, "vida");
        }
        _ => panic!("expected Died event"),
    }
    assert!(
        sim.world.creatures.iter().flatten().all(|c| c.id != id_a),
        "id_a debe haber desaparecido del almacén"
    );

    // Creature B: health 100, lifespan 5 -> dies by edad at end of tick 5.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_b = species_no_program("B", 100, 5);
    let id_b = world.place_creature(&sp_b, 0, 0).expect("place B");
    let mut sim = Simulation::new(world);

    for _ in 0..4 {
        let events = sim.step_tick();
        assert!(
            events.iter().all(|e| !matches!(e, Event::Died { .. })),
            "no death expected before tick 5"
        );
    }
    let creature = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id_b)
        .expect("creature alive after 4 ticks");
    assert_eq!(creature.age, 4);

    let events = sim.step_tick();
    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { .. }))
        .collect();
    assert_eq!(died.len(), 1);
    match &died[0] {
        Event::Died {
            tick,
            species,
            cause,
            ..
        } => {
            assert_eq!(*tick, 4);
            assert_eq!(species, "B");
            assert_eq!(cause, "edad");
        }
        _ => panic!("expected Died event"),
    }
    assert!(
        sim.world.creatures.iter().flatten().all(|c| c.id != id_b),
        "id_b debe haber desaparecido del almacén"
    );
}

// ---------------------------------------------------------------------------
// AC3: step_tick itera solo sobre ids vivos del almacén (huecos ignorados).
// ---------------------------------------------------------------------------

#[test]
fn test_sim_iterates_live_ids_only() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_no_program("Walker", 100, 1000);
    let id_a = world.place_creature(&sp, 0, 0).expect("place A");
    let id_b = world.place_creature(&sp, 1, 0).expect("place B");
    let id_c = world.place_creature(&sp, 2, 0).expect("place C");

    // Liberar el hueco del medio antes del tick: id_b deja de existir.
    world.remove_creature(id_b).expect("remove B");

    // El almacén tiene un hueco en id_b; los ids vivos son id_a e id_c.
    let live: Vec<CreatureId> = world
        .creatures
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| slot.as_ref().map(|_| CreatureId(i)))
        .collect();
    assert_eq!(live, vec![id_a, id_c]);

    let mut sim = Simulation::new(world);
    let events = sim.step_tick();

    // Ninguna muerte ni error; solo los vivos envejecen un tick.
    assert!(
        !events.iter().any(|e| matches!(e, Event::Died { .. })),
        "no muertes esperadas, got: {events:?}"
    );
    let after: Vec<(CreatureId, i64)> = sim
        .world
        .creatures
        .iter()
        .enumerate()
        .filter_map(|(i, slot)| slot.as_ref().map(|c| (CreatureId(i), c.age)))
        .collect();
    assert_eq!(after, vec![(id_a, 1), (id_c, 1)]);
    assert!(
        sim.world.creatures[id_b.0].is_none(),
        "el hueco de id_b sigue libre tras el tick"
    );
}

// ---------------------------------------------------------------------------
// AC6: terrain regenerates (capped), objects with reserve 0 vanish.
// ---------------------------------------------------------------------------

#[test]
fn test_terrain_regenerates_and_empty_objects_vanish() {
    // Map with grass everywhere and a rock at (2,2).
    let mut map = empty_map();
    map.rows[2][2] = '#';
    let rock = rock_object();
    let world = build_world(&map, &[grass_terrain()], std::slice::from_ref(&rock));
    assert!(matches!(world.content[2][2], CellContent::Object(0)));

    let mut sim = Simulation::new(world);
    // Agota el objeto y drena el terreno sobre la reserva del mundo.
    sim.world.objects[0].reserve = 0;
    sim.world.terrains[0].reserve = 0;

    let _ = sim.step_tick();

    // Terreno en (0,0) con reserve 0, regen 2 -> ahora 2 (capped at 100).
    assert_eq!(sim.world.terrains[0].reserve, 2);
    // Objeto en (2,2) con reserve 0 -> desaparecido al final del tick.
    assert!(matches!(sim.world.content[2][2], CellContent::Empty));
}

/// AC2: el daño y la regeneración de fin de tick leen y escriben la misma
/// reserva; regenerar sobre una copia intacta restauraría el valor original.
#[test]
fn test_terrain_damage_then_regen_consistent() {
    let map = empty_map();
    let world = build_world(&map, &[grass_terrain()], &[]);
    let mut sim = Simulation::new(world);

    // Daña el terreno de (2,2): 100 -> 70.
    sim.world
        .damage_cell_content(2, 2, 30)
        .expect("daña terreno de (2,2)");
    assert_eq!(sim.world.terrains[0].reserve, 70);

    // Fin de tick: regenera +2 sobre la reserva dañada: 72 (no 100).
    let _ = sim.step_tick();
    assert_eq!(
        sim.world.terrains[0].reserve, 72,
        "la regeneración parte de la reserva dañada"
    );

    // Segunda vuelta: daña de nuevo la misma reserva y regenera sobre ella.
    sim.world
        .damage_cell_content(2, 2, 30)
        .expect("daña terreno de (2,2) de nuevo");
    assert_eq!(sim.world.terrains[0].reserve, 42);
    let _ = sim.step_tick();
    assert_eq!(sim.world.terrains[0].reserve, 44);
}

/// AC3: un objeto dañado hasta 0 o menos desaparece al final del tick y
/// libera su casilla.
#[test]
fn test_object_exhausted_still_removed() {
    let mut map = empty_map();
    map.rows[2][2] = '#';
    let rock = rock_object();
    let world = build_world(&map, &[grass_terrain()], std::slice::from_ref(&rock));
    assert!(matches!(world.content[2][2], CellContent::Object(0)));
    let mut sim = Simulation::new(world);

    // Agota el objeto de (2,2) sobre la reserva del mundo: 100 -> 0.
    sim.world
        .damage_cell_content(2, 2, 100)
        .expect("agota objeto de (2,2)");
    assert_eq!(sim.world.objects[0].reserve, 0);

    let _ = sim.step_tick();

    assert!(
        matches!(sim.world.content[2][2], CellContent::Empty),
        "el objeto agotado desaparece y libera su casilla"
    );
}

// ---------------------------------------------------------------------------
// AC8: determinism and seed-dependent turn order.
// ---------------------------------------------------------------------------

fn walker_species() -> Rc<Species> {
    species_with_program(
        "Walker",
        100,
        1000,
        "move(random % 3 - 1, random % 3 - 1, 1)\ngoto start\n",
    )
}

fn two_walker_world() -> World {
    // 5x3 map, all grass. Walker A at (0,0), Walker B at (1,0).
    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
    };
    let mut world = build_world(&map, &[grass_terrain()], &[]);
    let sp = walker_species();
    let id_a = world.place_creature(&sp, 0, 0).expect("place A");
    let id_b = world.place_creature(&sp, 1, 0).expect("place B");
    assert_ne!(id_a, id_b);
    world
}

fn run_ticks(sim: &mut Simulation, n: u64) -> Vec<Event> {
    let mut all = Vec::new();
    for _ in 0..n {
        all.extend(sim.step_tick());
    }
    all
}

#[test]
fn test_same_seed_same_history() {
    let world = two_walker_world();
    let mut sim1 = Simulation::new(world.clone());
    sim1.set_rng_seed(42);
    let events1 = run_ticks(&mut sim1, 50);
    let map1 = sim1.world.draw().unwrap();

    let mut sim2 = Simulation::new(world);
    sim2.set_rng_seed(42);
    let events2 = run_ticks(&mut sim2, 50);
    let map2 = sim2.world.draw().unwrap();

    assert_eq!(events1, events2, "same seed must produce same events");
    assert_eq!(map1, map2, "same seed must produce same map");
}

// ---------------------------------------------------------------------------
// reproduce: la cría no juega el tick de nacimiento (AC4).
// ---------------------------------------------------------------------------

#[test]
fn test_reproduce_offspring_starts_at_start_next_tick() {
    // Madre: start: reproduce(0, 1, 5) goto start
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_with_program("Walker", 100, 1000, "reproduce(0, 1, 5)\ngoto start\n");
    let mother = world.place_creature(&sp, 1, 1).expect("place mother");
    let mut sim = Simulation::new(world);

    // Tick 0: la madre paría; la cría nace en (1,2) pero el snapshot de
    // step_tick la excluye de este tick: pc sigue en 0.
    let events = sim.step_tick();
    assert!(
        events.iter().all(|e| !matches!(e, Event::Died { .. })),
        "no muertes esperadas en el tick de nacimiento, got: {events:?}"
    );
    let child = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("debe existir la cría tras reproducir");
    assert_eq!((child.x, child.y), (1, 2));
    assert_eq!(child.health, 4, "vida = health (5) menos 1 por fin de tick");
    assert_eq!(child.age, 1, "edad 0 al nacer, +1 al final del tick");
    assert_eq!(child.state, CreatureState::Awake);
    assert_eq!(
        child.pc, 0,
        "aún no ha ejecutado: arranca en start: el tick siguiente"
    );
    assert!(child.variables.is_empty(), "la cría nace sin variables");

    // Tick 1: la cría ejecuta su primer turno desde start: (label + reproduce).
    let events = sim.step_tick();
    assert!(
        events.iter().all(|e| !matches!(e, Event::Died { .. })),
        "no muertes esperadas en el segundo tick, got: {events:?}"
    );
    let child = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id != mother)
        .expect("la cría sigue viva");
    assert_eq!(
        child.pc, 2,
        "la cría ejecutó start: y reproduce en su primer turno"
    );
    assert_eq!(child.age, 2);
    assert_eq!(child.health, 3);
    // Su propio reproduce abortó (4 no es > 5): no hay segunda cría.
    assert_eq!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .filter(|c| c.id != mother)
            .count(),
        1,
        "solo una cría (el segundo reproduce abortó por vida insuficiente o celda ocupada)"
    );
}

#[test]
fn test_seed_changes_turn_order() {
    // Two walkers: A at (0,0), B at (1,0). Both try to move right.
    // A wants (1,0) [occupied by B], B wants (2,0) [free].
    // If B moves first, (1,0) frees up and A can move there next turn.
    // If A moves first, A is blocked, B moves to (2,0).

    let world = two_walker_world();
    let mut sim1 = Simulation::new(world.clone());
    sim1.set_rng_seed(42);
    let events1 = run_ticks(&mut sim1, 50);
    let map1 = sim1.world.draw().unwrap();

    let mut sim2 = Simulation::new(world);
    sim2.set_rng_seed(99);
    let events2 = run_ticks(&mut sim2, 50);
    let map2 = sim2.world.draw().unwrap();

    // The maps should differ after 50 ticks with different seeds.
    assert!(
        map1 != map2 || events1 != events2,
        "different seeds should change turn order and thus the outcome:\nmap1:\n{map1}\nmap2:\n{map2}"
    );
}

#[test]
fn test_roar_rotates_to_previous_tick() {
    // El rugido emitido en el tick actual pasa a ser "del tick anterior" al
    // final del tick (sim.rs rotación), listo para que F2 lo lea con
    // roars_near/last_roar en el tick siguiente.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_no_program("Walker", 100, 1000);
    let id = world.place_creature(&sp, 1, 1).expect("place walker");
    let mut sim = Simulation::new(world);

    sim.world.add_roar(
        id,
        "Walker".to_string(),
        Pos { x: 1, y: 1 },
        "hola".to_string(),
    );
    assert_eq!(sim.world.roars_current.len(), 1);
    assert!(sim.world.roars_previous.is_empty());
    assert_eq!(sim.world.roars_current[0].tick, sim.world.ticks);

    let _events = sim.step_tick();

    // Tras el tick: rotados a previous, current vacío.
    assert!(sim.world.roars_current.is_empty(), "current se limpió");
    assert_eq!(sim.world.roars_previous.len(), 1, "pasaron a previous");
    let stored = &sim.world.roars_previous[0];
    assert_eq!(
        stored.tick, 0,
        "tick del rugido (tick_actual - 1 tras el avance)"
    );
    assert_eq!(stored.text, "hola");
    assert_eq!(stored.pos, Pos { x: 1, y: 1 });
    assert_eq!(stored.species, "Walker");

    // Un segundo paso sin nuevos rugidos vacía previous: un rugido solo está
    // disponible el tick siguiente al de su emisión, no persiste indefinido.
    let _events = sim.step_tick();
    assert!(
        sim.world.roars_previous.is_empty(),
        "sin rugidos del tick anterior"
    );
    assert!(sim.world.roars_current.is_empty());
}

// ---------------------------------------------------------------------------
// AC6: una criatura compilada lee percepciones propias y del mundo y las dice
// ---------------------------------------------------------------------------

#[test]
fn test_simulation_reads_self_and_world_perceptions() {
    // X (facción A) dice su especie, su facción y cuántos enemigos ve.
    // Dos criaturas de la facción B la rodean: total_enemies debe ser 2.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp_x = species_with_program_and_faction(
        "X",
        "A",
        100,
        1000,
        "say(species)\nsay(faction)\nsay(total_enemies)\nwait(1)\ngoto start\n",
    );
    let _id_x = world.place_creature(&sp_x, 0, 0).expect("place X");
    let sp_b = species_no_program("B", 100, 1000);
    let _id_b1 = world.place_creature(&sp_b, 1, 0).expect("place B 1");
    let _id_b2 = world.place_creature(&sp_b, 2, 0).expect("place B 2");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);
    let events = sim.step_tick();

    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "X"),
        "dice su especie, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "A"),
        "dice su facción, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "2"),
        "dice total_enemies (2), got: {said:?}"
    );
}

// ---------------------------------------------------------------------------
// AC7: semántica de texto de punta a punta
// ---------------------------------------------------------------------------

#[test]
fn test_text_semantics_kill_on_type_error() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    // T: compara textos -> 1 y 0, sobrevive.
    let sp_t = species_with_program_and_faction(
        "T",
        "A",
        100,
        1000,
        "say(\"hola\" == \"hola\")\nsay(\"hola\" == \"mundo\")\nwait(1)\ngoto start\n",
    );
    let _id_t = world.place_creature(&sp_t, 0, 1).expect("place T");

    // D1: texto donde espera un entero (argumento de acción) -> muere en la línea 2.
    let sp_d1 =
        species_with_program_and_faction("D1", "A", 100, 1000, "wait(\"hola\")\ngoto start\n");
    let _id_d1 = world.place_creature(&sp_d1, 1, 1).expect("place D1");

    // D2: texto + entero en aritmética -> muere en la línea 2.
    let sp_d2 =
        species_with_program_and_faction("D2", "A", 100, 1000, "say(1 + \"a\")\ngoto start\n");
    let _id_d2 = world.place_creature(&sp_d2, 2, 1).expect("place D2");

    // D3: texto en comparación de orden -> muere en la línea 2.
    let sp_d3 = species_with_program_and_faction(
        "D3",
        "A",
        100,
        1000,
        "if \"a\" < 5 goto start\ngoto start\n",
    );
    let _id_d3 = world.place_creature(&sp_d3, 3, 1).expect("place D3");

    // D4: texto como argumento numérico de move -> muere en la línea 2.
    let sp_d4 =
        species_with_program_and_faction("D4", "A", 100, 1000, "move(\"a\", 0, 1)\ngoto start\n");
    let _id_d4 = world.place_creature(&sp_d4, 4, 1).expect("place D4");

    // S: say(42) muestra el número como texto y sobrevive.
    let sp_s =
        species_with_program_and_faction("S", "A", 100, 1000, "say(42)\nwait(1)\ngoto start\n");
    let _id_s = world.place_creature(&sp_s, 0, 0).expect("place S");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(11);
    let events = sim.step_tick();

    // T dice "1" y "0": comparar dos textos da 1 o 0.
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "1"),
        "texto == texto da 1, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "0"),
        "texto != texto da 0, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "42"),
        "say muestra el número como texto, got: {said:?}"
    );

    // D1..D4 mueren por error de ejecución con su línea.
    let errors: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert_eq!(
        errors.len(),
        4,
        "cuatro errores de ejecución, got: {errors:?}"
    );
    for err in &errors {
        if let Event::RuntimeError { line, .. } = err {
            // Cuerpo: 6 líneas de cabecera + `start:` -> primera línea del
            // cuerpo es la 7.
            assert_eq!(*line, 7, "el error nombra su línea: {err:?}");
        }
    }

    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { cause, .. } if cause == "error"))
        .collect();
    assert_eq!(died.len(), 4, "cuatro muertes por error, got: {died:?}");
    for id in [_id_d1, _id_d2, _id_d3, _id_d4] {
        assert!(
            sim.world.creatures.iter().flatten().all(|c| c.id != id),
            "{id:?} debe estar muerta"
        );
    }
    // T y S siguen vivas.
    assert!(sim
        .world
        .creatures
        .iter()
        .flatten()
        .any(|c| c.species.name == "T"));
    assert!(sim
        .world
        .creatures
        .iter()
        .flatten()
        .any(|c| c.species.name == "S"));
}

// ---------------------------------------------------------------------------
// AC5: roars_near and last_roar
// ---------------------------------------------------------------------------

#[test]
fn test_roars_near_and_last_roar() {
    // Creature E (emitter) roars in tick 0. Creature L (listener) is within
    // vision and should hear it in tick 1.
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    let sp_e = species_with_program_and_faction(
        "E",
        "A",
        100,
        1000,
        "roar(\"hola mundo\")\nwait(100)\ngoto start\n",
    );
    let _id_e = world.place_creature(&sp_e, 0, 0).expect("place emitter");

    let sp_l = species_with_program_and_faction(
        "L",
        "A", // same faction so E is an ally, but we test roars not faction
        100,
        1000,
        "say(roars_near)\nsay(last_roar)\nmove(0, 0, 0)\ngoto start\n",
    );
    let _id_l = world
        .place_creature(&sp_l, 1, 0)
        .expect("place listener at distance 1");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);

    // Tick 0: E roars. L hears nothing yet (roar is in roars_current).
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    // In tick 0, L hasn't heard anything yet (roar goes to previous at end of tick).
    assert!(
        said.iter().any(|t| **t == "0"),
        "L hears 0 roars in tick 0, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| t.is_empty()),
        "L hears empty last_roar in tick 0, got: {said:?}"
    );

    // Tick 1: E's roar is now in roars_previous. L should hear it.
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "1"),
        "L hears 1 roar in tick 1, got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "hola mundo"),
        "L hears last_roar = 'hola mundo', got: {said:?}"
    );
}

// ---------------------------------------------------------------------------
// AC6: roar is heard one tick only; own roar excluded
// ---------------------------------------------------------------------------

#[test]
fn test_roar_is_heard_one_tick_only() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    // E roars once then sleeps; L roars every tick and reports what it
    // hears. E's roar must be heard by L in tick 1 (one tick after
    // emission) and not in tick 2. L must never hear its own roars.
    let sp_e = species_with_program_and_faction(
        "E",
        "A",
        100,
        1000,
        "roar(\"hola\")\nwait(100)\ngoto start\n",
    );
    let _id_e = world
        .place_creature(&sp_e, 0, 0)
        .expect("place emitter at (0,0)");

    let sp_l = species_with_program_and_faction(
        "L",
        "A",
        100,
        1000,
        "roar(\"propio\")\nsay(roars_near)\nsay(last_roar)\ngoto start\n",
    );
    let _id_l = world
        .place_creature(&sp_l, 1, 0)
        .expect("place listener at (1,0)");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(3);

    // Tick 0: E roars "hola", L roars "propio". No says yet (roar ends turn).
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.is_empty(),
        "no says in tick 0 (roar ends turn), got: {said:?}"
    );

    // Tick 1: E sleeps. L says roars_near=1 (hears E's roar, not its own),
    // last_roar="hola". Then L roars again.
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "1"),
        "L hears 1 roar in tick 1 (E's roar, own excluded), got: {said:?}"
    );
    assert!(
        said.iter().any(|t| **t == "hola"),
        "L hears last_roar='hola' in tick 1, got: {said:?}"
    );

    // Tick 2: E sleeps. E's roar from tick 0 rotated out. L's own roar from
    // tick 1 is in previous but excluded. roars_near=0, last_roar="".
    let events = sim.step_tick();
    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said.iter().any(|t| **t == "0"),
        "L hears 0 roars in tick 2 (E's roar expired, own excluded), got: {said:?}"
    );
    assert!(
        said.iter().any(|t| t.is_empty()),
        "L hears empty last_roar in tick 2, got: {said:?}"
    );
}

// ---------------------------------------------------------------------------
// AC8: near perceptions are deterministic
// ---------------------------------------------------------------------------

#[test]
fn test_near_perceptions_are_deterministic() {
    // Two creatures use enemy_dist/ally_dist to decide movement. Same seed
    // -> same history.
    fn make_world() -> World {
        let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
        let sp_a = species_with_program_and_faction(
            "A",
            "F1",
            100,
            1000,
            "say(enemy_dist)\nsay(ally_dist)\nwait(1)\ngoto start\n",
        );
        let sp_b = species_with_program_and_faction(
            "B",
            "F2",
            100,
            1000,
            "say(enemy_dist)\nwait(1)\ngoto start\n",
        );
        world.place_creature(&sp_a, 0, 0).expect("place A");
        world.place_creature(&sp_b, 3, 0).expect("place B");
        world
    }

    let world1 = make_world();
    let mut sim1 = Simulation::new(world1);
    sim1.set_rng_seed(42);
    let events1: Vec<Event> = (0..10).flat_map(|_| sim1.step_tick()).collect();
    let said1: Vec<String> = events1
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    let world2 = make_world();
    let mut sim2 = Simulation::new(world2);
    sim2.set_rng_seed(42);
    let events2: Vec<Event> = (0..10).flat_map(|_| sim2.step_tick()).collect();
    let said2: Vec<String> = events2
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(
        said1, said2,
        "same seed produces same said history:\n{said1:?}\n{said2:?}"
    );
}

// ---------------------------------------------------------------------------
// AC8: see/name con argumentos enteros corre sin error; argumento de texto mata
// ---------------------------------------------------------------------------

#[test]
fn test_see_with_text_argument_kills_creature() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp =
        species_with_program_and_faction("D", "A", 100, 1000, "if see(\"a\", 0) == 0 goto start\n");
    let _id = world.place_creature(&sp, 0, 0).expect("place D");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    let errors: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert_eq!(errors.len(), 1, "un error de ejecución, got: {errors:?}");
    for err in &errors {
        if let Event::RuntimeError { line, .. } = err {
            assert_eq!(*line, 7, "el error nombra su línea: {err:?}");
        }
    }

    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { cause, .. } if cause == "error"))
        .collect();
    assert_eq!(died.len(), 1, "la criatura muere por el error de ejecución");
}

#[test]
fn test_creature_uses_see_and_name() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);
    let sp = species_with_program_and_faction(
        "Watcher",
        "A",
        100,
        1000,
        "say(see(1, 0))\nsay(name(1, 0))\nwait(1)\ngoto start\n",
    );
    let _id = world.place_creature(&sp, 0, 0).expect("place watcher");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(7);
    let events = sim.step_tick();

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::RuntimeError { .. })),
        "sin errores de ejecución, got: {events:?}"
    );
    assert!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .any(|c| c.species.name == "Watcher"),
        "la criatura sigue viva"
    );

    let said: Vec<&String> = events
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(said.len() >= 2, "dice see y name, got: {said:?}");
}

#[test]
fn test_simulation_with_injected_rng_is_deterministic() {
    // AC3: Un TestRng inyectado controla shuffle (orden de turnos) y
    // next_bounded (percepción random). Dos corridas con el mismo TestRng
    // producen la misma historia.
    // TestRng con valores fijos: random devuelve 0, 1, 2 en secuencia.
    // La criatura dice random cada turno, así que el TestRng controla la salida.
    fn make_world() -> World {
        let map = Map {
            name: "test".to_string(),
            rows: vec![vec!['.'; 5], vec!['.'; 5], vec!['.'; 5]],
        };
        let terrain = Terrain {
            name: "Grass".to_string(),
            character: '.',
            resource_max: 100,
            regen: 2,
            reserve: 100,
        };
        let mut world = World::from_map(&map, &[terrain], &[]);
        assert!(world.diagnostics.is_empty());
        let sp = species_with_program("Walker", 100, 1000, "say(random)\nwait(1)\ngoto start\n");
        let _id = world.place_creature(&sp, 0, 0).expect("place walker");
        world
    }

    // TestRng con valores fijos: random devuelve 0, 1, 2 en secuencia.
    let mut world1 = make_world();
    world1.rng = RefCell::new(Box::new(TestRng::new(vec![0, 1, 2])));
    let mut sim1 = Simulation::new(world1);
    let events1 = run_ticks(&mut sim1, 10);

    let mut world2 = make_world();
    world2.rng = RefCell::new(Box::new(TestRng::new(vec![0, 1, 2])));
    let mut sim2 = Simulation::new(world2);
    let events2 = run_ticks(&mut sim2, 10);

    assert_eq!(
        events1, events2,
        "same injected TestRng produces same history:\n{events1:?}\n{events2:?}"
    );

    let said1: Vec<&String> = events1
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said1.iter().any(|t| **t == "0"),
        "dice random=0, got: {said1:?}"
    );
    assert!(
        said1.iter().any(|t| **t == "1"),
        "dice random=1, got: {said1:?}"
    );
    assert!(
        said1.iter().any(|t| **t == "2"),
        "dice random=2, got: {said1:?}"
    );

    // Un TestRng diferente produce una historia diferente (controla random).
    let mut world3 = make_world();
    world3.rng = RefCell::new(Box::new(TestRng::new(vec![5, 5, 5])));
    let mut sim3 = Simulation::new(world3);
    let events3 = run_ticks(&mut sim3, 10);

    let said3: Vec<&String> = events3
        .iter()
        .filter_map(|e| match e {
            Event::Said { text, .. } => Some(text),
            _ => None,
        })
        .collect();
    assert!(
        said3.iter().all(|t| **t == "5"),
        "dice random=5 siempre, got: {said3:?}"
    );
    assert_ne!(
        events1, events3,
        "different injected TestRng produces different history"
    );
}

// ---------------------------------------------------------------------------
// Memento: turno fallido no corrompe el mundo ni filtra variables
// ---------------------------------------------------------------------------

/// Una criatura ejecuta `set("x", 1)` y luego una acción con argumento de
/// texto que falla (error de ejecución). La criatura debe morir limpiamente
/// y el mundo no debe quedar corrupto: las otras criaturas siguen vivas.
#[test]
fn test_turn_failure_rolls_back_or_dies_cleanly() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    // Criatura que falla: contador = 1 luego wait con texto -> error de ejecución.
    let sp_fail = species_with_program_and_faction(
        "Failing",
        "A",
        100,
        1000,
        "contador = 1\nwait(\"no es un numero\")\ngoto start\n",
    );
    let id_fail = world
        .place_creature(&sp_fail, 0, 0)
        .expect("place failing creature");

    // Otra criatura sana que debe seguir viva después del error.
    let sp_ok = species_no_program("Ok", 100, 1000);
    let id_ok = world
        .place_creature(&sp_ok, 1, 0)
        .expect("place ok creature");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let events = sim.step_tick();

    // Debe haber un RuntimeError y un Died por causa "error".
    let errors: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert_eq!(errors.len(), 1, "un error de ejecución, got: {errors:?}");

    let died: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::Died { cause, .. } if cause == "error"))
        .collect();
    assert_eq!(died.len(), 1, "una muerte por error, got: {died:?}");

    // La criatura fallida debe haber desaparecido del almacén.
    assert!(
        sim.world
            .creatures
            .iter()
            .flatten()
            .all(|c| c.id != id_fail),
        "la criatura fallida debe estar muerta"
    );

    // La criatura sana sigue viva y sin cambios inesperados.
    let ok = sim
        .world
        .creatures
        .iter()
        .flatten()
        .find(|c| c.id == id_ok)
        .expect("la criatura sana sigue viva");
    assert_eq!(ok.health, 99); // fin de tick: 100 - 1 salud
    assert_eq!(ok.age, 1); // fin de tick: +1 edad

    // No debe haber huérfanos en la rejilla.
    for row in &sim.world.content {
        for cell in row {
            if let CellContent::Creature(id) = cell {
                assert!(
                    sim.world.creatures[id.0].is_some(),
                    "la rejilla no debe referenciar criatura inexistente {id:?}"
                );
            }
        }
    }
}

/// Una criatura asigna una variable y luego falla. Tras el turno fallido,
/// la variable no debe "filtrarse" parcialmente: la criatura está muerta,
/// por lo que su mapa de variables ya no existe en el mundo.
#[test]
fn test_variables_not_leaked_on_failed_turn() {
    let mut world = build_world(&empty_map(), &[grass_terrain()], &[]);

    let sp = species_with_program_and_faction(
        "Leaker",
        "A",
        100,
        1000,
        "tesoro = 42\noro = 7\nattack(\"a\", 0, 1)\ngoto start\n",
    );
    let id = world.place_creature(&sp, 0, 0).expect("place leaker");

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(1);
    let _events = sim.step_tick();

    // La criatura debe haber muerto por error de ejecución.
    assert!(
        sim.world.creatures.iter().flatten().all(|c| c.id != id),
        "la criatura debe estar muerta tras el turno fallido"
    );

    // No debe haber ninguna criatura viva con la variable "tesoro" (la
    // criatura que la asignó ya no existe).
    let leaked: Vec<CreatureId> = sim
        .world
        .creatures
        .iter()
        .flatten()
        .filter(|c| c.variables.contains_key("tesoro"))
        .map(|c| c.id)
        .collect();
    assert!(
        leaked.is_empty(),
        "ninguna criatura viva debe tener la variable 'tesoro', got: {leaked:?}"
    );
}
