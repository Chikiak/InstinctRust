//! Random perception tests.

use super::common::*;
use instinct_core::actions::Catalog;
use instinct_core::expr::parse;
use instinct_core::interp::InterpreterVisitor;
use instinct_core::perceptions::{CreatureView, Perception, PerceptionCatalog};
use instinct_core::resources::{Map, Object, Terrain};
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::{CreatureContext, CreatureId, World};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[test]
fn test_random_consumes_one_draw_per_read() {
    let mut world = empty_world();
    let id = place_walker(&mut world);

    world.set_rng_seed(42);
    let d1 = unwrap_int(eval(&world, id, "random").unwrap());
    let d2 = unwrap_int(eval(&world, id, "random").unwrap());
    let d3 = unwrap_int(eval(&world, id, "random").unwrap());

    // Same seed reproduces the same sequence.
    world.set_rng_seed(42);
    let e1 = unwrap_int(eval(&world, id, "random").unwrap());
    let e2 = unwrap_int(eval(&world, id, "random").unwrap());
    assert_eq!(e1, d1);
    assert_eq!(e2, d2);

    // Two reads in one expression consume two draws: random + random == d1 + d2,
    // and the following read is d3 (not d2).
    world.set_rng_seed(42);
    let sum = unwrap_int(eval(&world, id, "random + random").unwrap());
    let next = unwrap_int(eval(&world, id, "random").unwrap());
    assert_eq!(sum, d1 + d2);
    assert_eq!(next, d3);
}
