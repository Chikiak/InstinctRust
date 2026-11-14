//! Aggregated nearby information for a creature: counts, nearest of each
//! class, and roar data from the previous tick.

/// The nearest target of a given class: Chebyshev distance and normalized
/// direction (dx, dy in {-1, 0, 1}).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nearest {
    pub dist: i64,
    pub dx: i64,
    pub dy: i64,
}

/// Aggregated nearby information for a creature: counts, nearest of each
/// class, and roar data from the previous tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nearby {
    pub allies: i64,
    pub enemies: i64,
    pub objects: i64,
    pub ally: Option<Nearest>,
    pub enemy: Option<Nearest>,
    pub object: Option<Nearest>,
    pub roars: i64,
    pub last_roar: String,
}

impl Nearby {
    /// Empty nearby: no allies, enemies, objects, or roars within radius.
    pub fn empty() -> Self {
        Nearby {
            allies: 0,
            enemies: 0,
            objects: 0,
            ally: None,
            enemy: None,
            object: None,
            roars: 0,
            last_roar: String::new(),
        }
    }
}
