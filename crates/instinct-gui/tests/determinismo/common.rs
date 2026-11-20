//! Shared helpers for `determinismo` tests.

/// Root de `examples`, resuelto desde el manifest dir del crate.
pub fn examples_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// Semilla usada en la puerta de determinismo (F5-08 AC2).
pub const DETERMINISMO_SEED: u64 = 7;

/// Número de ticks comparados en la puerta de determinismo.
pub const DETERMINISMO_TICKS: u64 = 5;

/// Criaturas y coordenadas del baseline de determinismo (plan F5-08).
pub fn baseline_criaturas() -> Vec<(&'static str, usize, usize)> {
    vec![
        ("Dwarf", 1, 0),
        ("Ent", 5, 1),
        ("Hobbit", 10, 0),
        ("Uruk", 12, 3),
    ]
}
