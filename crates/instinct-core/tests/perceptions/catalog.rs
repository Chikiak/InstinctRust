//! Catalog extension perception tests.

use instinct_core::actions::Catalog;
use instinct_core::perceptions::{CreatureView, Perception, PerceptionCatalog};
use instinct_core::value::Value;

#[derive(Debug, Clone, Copy)]
struct Bonus;

impl Perception for Bonus {
    fn name(&self) -> &str {
        "bonus"
    }
    fn read(&self, _view: &dyn CreatureView) -> Value {
        Value::Int(7)
    }
    fn clone_box(&self) -> Box<dyn Perception> {
        Box::new(*self)
    }
}

#[derive(Debug)]
struct EmptyView;

impl CreatureView for EmptyView {
    fn health(&self) -> i64 {
        0
    }
    fn age(&self) -> i64 {
        0
    }
    fn lifespan(&self) -> i64 {
        0
    }
    fn x(&self) -> i64 {
        0
    }
    fn y(&self) -> i64 {
        0
    }
    fn tick(&self) -> u64 {
        0
    }
    fn random(&self) -> i64 {
        0
    }
    fn vision(&self) -> i64 {
        0
    }
    fn species_name(&self) -> &str {
        ""
    }
    fn faction_name(&self) -> &str {
        ""
    }
    fn terrain_here(&self) -> &str {
        ""
    }
    fn reserve_here(&self) -> i64 {
        0
    }
    fn width(&self) -> i64 {
        0
    }
    fn height(&self) -> i64 {
        0
    }
    fn total_creatures(&self) -> i64 {
        0
    }
    fn total_allies(&self) -> i64 {
        0
    }
    fn total_enemies(&self) -> i64 {
        0
    }
    fn nearby(&self) -> instinct_core::perceptions::Nearby {
        instinct_core::perceptions::Nearby::empty()
    }

    fn see(&self, _x: i64, _y: i64) -> i64 {
        0
    }

    fn name(&self, _x: i64, _y: i64) -> &str {
        ""
    }
}

#[test]
fn test_catalog_bonus_absent_and_builtins_present() {
    let cat = Catalog::default();
    assert!(
        cat.resolve_perception("bonus").is_none(),
        "bonus not registered yet"
    );
    // The seven built-in perceptions are still present.
    for name in ["health", "age", "lifespan", "x", "y", "tick", "random"] {
        assert!(
            cat.resolve_perception(name).is_some(),
            "{name} still registered"
        );
    }
}

#[test]
fn test_catalog_register_bonus_reads_seven() {
    let mut cat = Catalog::default();
    assert!(
        cat.register_perception(Box::new(Bonus)).is_ok(),
        "register bonus"
    );
    let view = EmptyView;
    let p = cat.resolve_perception("bonus").expect("bonus resolves");
    assert_eq!(p.read(&view), Value::Int(7));
}

#[test]
fn test_default_catalogs_resolve_self_and_world_perceptions() {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    for name in [
        "vision",
        "species",
        "faction",
        "terrain_here",
        "reserve_here",
        "width",
        "height",
        "total_creatures",
        "total_allies",
        "total_enemies",
    ] {
        assert!(
            cat.resolve_perception(name).is_some(),
            "Catalog::default() must register {name}"
        );
        assert!(
            pc.resolve(name).is_some(),
            "PerceptionCatalog::default() must register {name}"
        );
    }
}

#[test]
fn test_default_catalogs_resolve_all_31_perceptions() {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let all_31 = [
        // 7 original (F0)
        "health",
        "age",
        "lifespan",
        "x",
        "y",
        "tick",
        "random",
        // 10 self/world (F3-01)
        "vision",
        "species",
        "faction",
        "terrain_here",
        "reserve_here",
        "width",
        "height",
        "total_creatures",
        "total_allies",
        "total_enemies",
        // 14 nearby (F3-02)
        "allies_near",
        "enemies_near",
        "objects_near",
        "enemy_dist",
        "enemy_dx",
        "enemy_dy",
        "ally_dist",
        "ally_dx",
        "ally_dy",
        "object_dist",
        "object_dx",
        "object_dy",
        "roars_near",
        "last_roar",
    ];
    for name in all_31 {
        assert!(
            cat.resolve_perception(name).is_some(),
            "Catalog::default() must register {name}"
        );
        assert!(
            pc.resolve(name).is_some(),
            "PerceptionCatalog::default() must register {name}"
        );
    }
}
