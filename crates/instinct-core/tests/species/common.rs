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
    // AC2: `Species` is immutable intrinsic shared data (Flyweight) and
    // `program` is `Option<Rc<Program>>`; two species may share the same
    // program by pointer.
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

    // Fields are read but there are no setters: the species is not mutated
    // after registration (no `set_*` methods).
    assert_eq!(grass.name, "Grass");
    assert_eq!(grass.faction, "Forest");
    assert_eq!(grass.health, 20);
    assert_eq!(grass.vision, 5);
    assert_eq!(grass.lifespan, 100);

    // The program is shared by `Rc`: same pointer in both species.
    assert!(Rc::ptr_eq(
        grass.program.as_ref().unwrap(),
        same_program.program.as_ref().unwrap()
    ));
    assert!(Rc::ptr_eq(grass.program.as_ref().unwrap(), &program));

    // Without cloning `Species`: two distinct species are distinct pointers.
    assert!(!Rc::ptr_eq(&grass, &same_program));
}
