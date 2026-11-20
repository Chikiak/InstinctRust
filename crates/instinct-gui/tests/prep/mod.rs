//! Integration tests for `app::prep`: headless placement and removal of
//! creatures. No window, no eframe.
//!
//! Pattern repo: `tests/<area>/mod.rs` with `mod` declarations and re-exports.

mod common;

use instinct_gui::app::prep::PlacementId;

use crate::common::PrepState;
use crate::common::{enter_prep_examples, examples_root, first_species_name, LoadedSession};

/// t-prep-01: `enter_prep` over a valid map yields an empty prep state.
#[test]
fn t_prep_01_enter_prep_valid_map() {
    let session = LoadedSession::new(&examples_root());
    let prep = PrepState::enter_prep(&session, "Plain with rocks", 7).expect("enter_prep ok");

    assert_eq!(prep.recipe.map_name, "Plain with rocks");
    assert_eq!(prep.recipe.seed, 7);
    assert_eq!(prep.placed_count(), 0);
    assert!(prep.creatures_in_prep().is_empty());
    assert!(prep.world.cell_is_free(0, 0));
}

/// t-prep-02: placing a creature on a free cell succeeds; world reflects it.
#[test]
fn t_prep_02_place_valid_creature() {
    let mut prep = enter_prep_examples();
    let species_name = first_species_name(&LoadedSession::new(&examples_root()));

    let id = prep
        .place(&LoadedSession::new(&examples_root()), &species_name, 1, 0)
        .expect("place ok");

    assert_eq!(prep.placed_count(), 1);
    assert_eq!(id, PlacementId(0));
    assert!(prep.occupied_cell(1, 0), "cell (1,0) occupied after place");
    let creatures = prep.creatures_in_prep();
    assert_eq!(creatures.len(), 1);
    assert_eq!(creatures[0], (PlacementId(0), species_name.clone(), 1, 0));
}

/// t-prep-03: unknown species → Diagnostic, no panic.
#[test]
fn t_prep_03_unknown_species_diagnostic() {
    let mut prep = enter_prep_examples();

    let result = prep.place(&LoadedSession::new(&examples_root()), "NoSuchSpecies", 1, 0);
    assert!(result.is_err(), "unknown species must error");
    assert_eq!(prep.placed_count(), 0, "no placement recorded on failure");
}

/// t-prep-04: placing on an object → Diagnostic, no panic.
#[test]
fn t_prep_04_place_on_object() {
    let mut prep = enter_prep_examples();
    let species_name = first_species_name(&LoadedSession::new(&examples_root()));

    // Find an object cell: row 1, col 3 is '#' in "Plain with rocks".
    assert!(prep.occupied_cell(3, 1), "cell (3,1) should hold an object");

    let result = prep.place(&LoadedSession::new(&examples_root()), &species_name, 3, 1);
    assert!(result.is_err(), "placing on object must error");
    assert_eq!(prep.placed_count(), 0);
}

/// t-prep-05: placing on an occupied cell → Diagnostic, no panic.
#[test]
fn t_prep_05_place_on_creature() {
    let mut prep = enter_prep_examples();
    let session = LoadedSession::new(&examples_root());
    let species_name = first_species_name(&session);

    prep.place(&session, &species_name, 5, 5)
        .expect("first place ok");
    assert!(prep.occupied_cell(5, 5), "cell (5,5) now occupied");

    let result = prep.place(&session, &species_name, 5, 5);
    assert!(result.is_err(), "placing on creature must error");
    assert_eq!(prep.placed_count(), 1, "only one placement recorded");
}

/// t-prep-06: placing out of bounds → Diagnostic, no panic.
#[test]
fn t_prep_06_place_out_of_bounds() {
    let mut prep = enter_prep_examples();
    let session = LoadedSession::new(&examples_root());
    let species_name = first_species_name(&session);

    let result = prep.place(&session, &species_name, 9999, 9999);
    assert!(result.is_err(), "out of bounds must error");
    assert_eq!(prep.placed_count(), 0);
}

/// t-prep-07: removing a placed creature frees the cell.
#[test]
fn t_prep_07_remove_valid() {
    let mut prep = enter_prep_examples();
    let session = LoadedSession::new(&examples_root());
    let species_name = first_species_name(&session);

    let id = prep.place(&session, &species_name, 2, 2).expect("place ok");
    assert!(!prep.world.cell_is_free(2, 2), "cell occupied after place");

    prep.remove(id).expect("remove ok");
    assert_eq!(prep.placed_count(), 0, "recipe cleared");
    assert!(prep.world.cell_is_free(2, 2), "cell free after remove");
    assert!(prep.creatures_in_prep().is_empty());
}

/// t-prep-08: removing a nonexistent placement id → Diagnostic, no panic.
#[test]
fn t_prep_08_remove_nonexistent() {
    let mut prep = enter_prep_examples();

    let result = prep.remove(PlacementId(42));
    assert!(result.is_err(), "nonexistent id must error");
    assert_eq!(prep.placed_count(), 0);
}

/// t-prep-09: placing N creatures of the same species works; world count reflects it.
#[test]
fn t_prep_09_many_same_species() {
    let mut prep = enter_prep_examples();
    let session = LoadedSession::new(&examples_root());
    let species_name = first_species_name(&session);

    let n = 5;
    let mut ids = Vec::new();
    for i in 0..n {
        let id = prep.place(&session, &species_name, i, 0).expect("place ok");
        ids.push(id);
    }

    assert_eq!(prep.placed_count(), n);
    assert_eq!(prep.world.live_creature_ids().len(), n);
    for (i, _id) in ids.iter().enumerate() {
        assert!(
            prep.occupied_cell(i, 0),
            "cell ({}, 0) occupied after place",
            i
        );
    }
}

/// t-prep-10: re-entering prep resets placements.
#[test]
fn t_prep_10_re_enter_prep() {
    let mut prep = enter_prep_examples();
    let session = LoadedSession::new(&examples_root());
    let species_name = first_species_name(&session);

    prep.place(&session, &species_name, 1, 1).expect("place ok");
    assert_eq!(prep.placed_count(), 1);

    let prep2 = PrepState::enter_prep(&session, "Plain with rocks", 3).expect("re-enter ok");
    assert_eq!(prep2.placed_count(), 0);
    assert_eq!(prep2.recipe.seed, 3);
    assert!(prep2.world.cell_is_free(1, 1));
}

/// t-prep-11: `occupied_cell` reports occupancy correctly.
#[test]
fn t_prep_11_occupied_cell() {
    let mut prep = enter_prep_examples();
    let session = LoadedSession::new(&examples_root());
    let species_name = first_species_name(&session);

    assert!(!prep.occupied_cell(4, 4), "free cell initially");
    // Out of bounds counts as occupied (not free).
    assert!(prep.occupied_cell(9999, 9999), "out of bounds is occupied");

    prep.place(&session, &species_name, 4, 4).expect("place ok");
    assert!(prep.occupied_cell(4, 4), "occupied after place");

    prep.remove(PlacementId(0)).expect("remove ok");
    assert!(!prep.occupied_cell(4, 4), "free again after remove");
}
