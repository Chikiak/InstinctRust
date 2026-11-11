//! Unit tests for the `species` module.

use instinct_core::program::Program;
use instinct_core::species::Species;
use std::rc::Rc;

#[test]
fn test_species_new_stores_name() {
    let s = Species::new("Walker");
    assert_eq!(s.name, "Walker");
}

#[test]
fn test_species_clone_and_eq() {
    let a = Species::new("Walker");
    let b = a.clone();
    assert_eq!(a, b);
    assert_ne!(a, Species::new("Sleeper"));
}

#[test]
fn test_species_is_immutable_shared() {
    // AC2: `Species` es dato intrínseco inmutable compartido (Flyweight) y
    // `program` es `Option<Rc<Program>>`; dos especies pueden compartir el
    // mismo programa por puntero.
    let program = Rc::new(Program::new());
    let grass = Rc::new(Species::with_program(
        "Grass",
        "Forest",
        20,
        5,
        100,
        Rc::clone(&program),
    ));
    let same_program = Rc::new(Species::with_program(
        "Grass2",
        "Forest",
        20,
        5,
        100,
        Rc::clone(&program),
    ));

    // Los campos se leen pero no hay setters: la especie no se muta tras
    // registrarla (no hay métodos `set_*`).
    assert_eq!(grass.name, "Grass");
    assert_eq!(grass.faction, "Forest");
    assert_eq!(grass.health, 20);
    assert_eq!(grass.vision, 5);
    assert_eq!(grass.lifespan, 100);

    // El programa se comparte por `Rc`: mismo puntero en ambas especies.
    assert!(Rc::ptr_eq(
        grass.program.as_ref().unwrap(),
        same_program.program.as_ref().unwrap()
    ));
    assert!(Rc::ptr_eq(grass.program.as_ref().unwrap(), &program));

    // Sin `Species` clonado: dos especies distintas son punteros distintos.
    assert!(!Rc::ptr_eq(&grass, &same_program));
}
