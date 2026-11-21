// Pruebas de escenario F6-02 (núcleo): golpes, consumo y reservas sobre el
// estado interno del mundo tras los ticks.
//
// Nota sobre las coordenadas: el mapa `Arena` es 9x9 (x en 0..9, y en 0..9,
// indices 0..=8 en ambos ejes). Las pruebas de "fuera del mapa" colocan a la
// criatura en la ultima columna valida (8) y hacen que su acción apunte a
// x = 9 (fuera del mapa), ya que colocar una criatura en x = 9 fallaria el
// propio `place_creature`.

use super::BankWorld;
use instinct_core::world::CellContent;

/// Reserva del objeto en la celda (x, y), si la hay.
fn object_reserve_at(world: &instinct_core::world::World, x: usize, y: usize) -> Option<i64> {
    match &world.content[y][x] {
        CellContent::Object(obj_idx) => Some(world.objects[*obj_idx].reserve),
        _ => None,
    }
}

/// Reserva del terreno en la celda (x, y).
fn terrain_reserve_at(world: &instinct_core::world::World, x: usize, y: usize) -> i64 {
    let t_idx = world.terrain[y][x];
    world.terrains[t_idx].reserve
}

// AC2: herida sin matar (Tough: 20 - 5 - 1 fin de tick = 14, sigue viva).
#[test]
fn attack_wounds_without_killing() {
    let mut bw = BankWorld::from_examples("Arena", &[("Bruiser", 2, 6), ("Tough", 3, 6)]);
    bw.step_tick();
    let tough = bw.memento("Tough").expect("Tough viva tras la herida");
    assert_eq!(
        tough.health, 14,
        "Tough: 20 - 5 (golpe) - 1 (fin de tick) = 14"
    );
    assert!(bw.memento("Tough").is_some(), "Tough sigue viva");
}

// AC3: ataque reduce la reserva del objeto (roca 100 -> 90).
#[test]
fn attack_reduces_object_reserve() {
    let mut bw = BankWorld::from_examples("Arena", &[("Miner", 3, 4)]);
    bw.step_tick();
    assert_eq!(
        object_reserve_at(&bw.sim.world, 4, 4),
        Some(90),
        "roca en (4,4): 100 - 10 = 90"
    );
}

// AC4: ataque reduce la reserva del terreno (hierba dañada y regenerada).
#[test]
fn attack_reduces_terrain_reserve() {
    let mut bw = BankWorld::from_examples("Arena", &[("Miner", 2, 2)]);
    bw.step_tick();
    let world = &bw.sim.world;
    // Hierba: 100 -> 90 por el golpe, luego +2 de regeneracion al final del
    // tick = 92 (coherente con AC8).
    assert_eq!(
        terrain_reserve_at(world, 3, 2),
        92,
        "hierba en (3,2): dañada y regenerada a 92"
    );
    // La celda sigue siendo hierba: contenido vacio y terreno caracter '.'.
    // Indices en [y][x] (coherente con object_reserve_at / terrain_reserve_at).
    assert_eq!(
        world.content[2][3],
        CellContent::Empty,
        "(3,2) sigue siendo terreno"
    );
    assert_eq!(
        world.terrains[world.terrain[2][3]].character, '.',
        "(3,2) sigue siendo hierba"
    );
}

// AC5: ataque a la propia celda golpea el terreno bajo la criatura (no a la criatura).
#[test]
fn attack_own_cell_hits_terrain() {
    let mut bw = BankWorld::from_examples("Arena", &[("Grazer", 6, 6)]);
    bw.step_tick();
    let grazer = bw.memento("Grazer").expect("Grazer viva");
    assert_eq!(
        grazer.health, 19,
        "Grazer: 20 - 1 (fin de tick), no dañado por el ataque"
    );
    assert_eq!(grazer.age, 1, "Grazer envejece 1");
    let world = &bw.sim.world;
    assert_eq!(
        terrain_reserve_at(world, 6, 6),
        92,
        "hierba en (6,6): dañada y regenerada a 92"
    );
}

// AC7: consume cura por encima de la salud de cabecera (sin tope).
#[test]
fn consume_heals_above_header_health() {
    let mut bw = BankWorld::from_examples("Arena", &[("Forager", 2, 8)]);
    bw.step_tick();
    let forager = bw.memento("Forager").expect("Forager viva");
    assert_eq!(
        forager.health, 29,
        "Forager: 10 + 20 (curar) - 1 (fin de tick) = 29"
    );
}

// AC6: consume fuera del mapa no cambia nada (sin curar, sin eventos).
#[test]
fn consume_out_of_map_changes_nothing() {
    let mut bw = BankWorld::from_examples("Arena", &[("Forager", 8, 8)]);
    let events = bw.step_tick();
    let world = &bw.sim.world;
    assert!(
        events.is_empty(),
        "consume fuera del mapa no debe emitir eventos, got: {events:?}"
    );
    assert_eq!(
        object_reserve_at(world, 4, 4),
        Some(100),
        "roca en (4,4) intacta (consume fuera del mapa no golpea)"
    );
    assert_eq!(
        terrain_reserve_at(world, 8, 8),
        100,
        "hierba en (8,8) intacta"
    );
    let forager = bw.memento("Forager").expect("Forager viva");
    assert_eq!(
        forager.health, 9,
        "Forager: 10 - 1 (fin de tick), sin curar al no haber eventos"
    );
}

// AC8: leyes de fin de tick.
#[test]
fn end_of_tick_laws() {
    // Grazer: ataque a propia celda daña hierba (100 -> 90 -> 92 tras regenerar),
    // -1 salud y +1 edad al final del tick.
    {
        let mut bw = BankWorld::from_examples("Arena", &[("Grazer", 6, 6)]);
        bw.step_tick();
        let world = &bw.sim.world;
        assert_eq!(
            terrain_reserve_at(world, 6, 6),
            92,
            "hierba en (6,6) regenerada a 92 tras el tick"
        );
        let grazer = bw.memento("Grazer").expect("Grazer viva");
        assert_eq!(grazer.health, 19, "Grazer salud tras fin de tick");
        assert_eq!(grazer.age, 1, "Grazer edad tras fin de tick");
    }

    // Miner@3,4 golpea la roca en (4,4). Como el programa es una sola acción,
    // el pc llega al final del archivo y reinicia en `start`, por lo que el
    // Miner ataca un turno sí y otro no: 10 golpes (ticks impares) agotan la
    // roca (100 -> 0) al final del tick 19, y el objeto se elimina.
    {
        let mut bw = BankWorld::from_examples("Arena", &[("Miner", 3, 4)]);
        bw.step_tick(); // tick 1: primer golpe
        assert_eq!(
            object_reserve_at(&bw.sim.world, 4, 4),
            Some(90),
            "roca en (4,4): 90 tras el tick 1"
        );
        // Tick 2 no ataca (reinicio de pc); los golpes siguientes caen en
        // ticks impares. Total 19 ticks = 10 golpes -> roca agotada.
        bw.step_ticks(18); // ticks 2..19, total 19
        assert_eq!(
            object_reserve_at(&bw.sim.world, 4, 4),
            None,
            "roca en (4,4) agotada y eliminada tras 19 ticks"
        );
        assert_eq!(
            bw.sim.world.content[4][4],
            CellContent::Empty,
            "celda (4,4) liberada"
        );
    }

    // Cualquier criatura: -1 salud y +1 edad por tick.
    {
        let mut bw = BankWorld::from_examples("Arena", &[("Brawler", 2, 2)]);
        bw.step_tick();
        let brawler = bw.memento("Brawler").expect("Brawler viva");
        assert_eq!(brawler.health, 19, "criatura: 20 - 1 = 19");
        assert_eq!(brawler.age, 1, "criatura: 0 + 1 = 1");
    }
}
