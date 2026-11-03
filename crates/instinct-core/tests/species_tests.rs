//! Unit tests for the `species` module.

use instinct_core::species::Species;

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
