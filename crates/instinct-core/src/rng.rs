//! Generador pseudoaleatorio propio y sembrado.
//!
//! Algoritmo: SplitMix64. Es una permutación de 64 bits simple, rápida y
//! determinista, independiente de la plataforma y de la versión de Rust (no
//! usa `usize` en ninguna operación de generación). Cada llamada a `next_u64`
//! avanza el estado interno y devuelve un nuevo valor.

/// Generador pseudoaleatorio con semilla de 64 bits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

const SPLITMIX64_INCREMENT: u64 = 0x9E3779B97F4A7C15;
const SPLITMIX64_MUL1: u64 = 0xBF58476D1CE4E5B9;
const SPLITMIX64_MUL2: u64 = 0x94D049BB133111EB;

impl Rng {
    /// Construye un generador desde una semilla de 64 bits (la semilla 0 funciona).
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// Avanza el estado y devuelve el siguiente u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX64_INCREMENT);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX64_MUL1);
        z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX64_MUL2);
        z ^ (z >> 31)
    }

    /// Devuelve un entero entre 0 y `max` inclusive (máximo incluido).
    /// `max` debe ser >= 0; si es negativo se trata como 0.
    pub fn next_bounded(&mut self, max: i64) -> i64 {
        let max = if max < 0 { 0 } else { max };
        let range = (max as u64).wrapping_add(1);
        (self.next_u64() % range) as i64
    }

    /// Baraja `lista` in-place usando Fisher–Yates con este mismo generador.
    /// Conserva todos los elementos y con la misma semilla produce el mismo orden.
    pub fn shuffle<T>(&mut self, lista: &mut [T]) {
        let mut i = lista.len();
        while i > 1 {
            i -= 1;
            let j = self.next_bounded(i as i64) as usize;
            lista.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_is_deterministic() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..1000 {
            assert_eq!(a.next_u64(), b.next_u64());
            assert_eq!(a.next_bounded(99), b.next_bounded(99));
        }
    }

    #[test]
    fn test_rng_different_seeds_different_sequences() {
        let mut a = Rng::new(0);
        let mut b = Rng::new(1);
        let mut diferente = false;
        for _ in 0..100 {
            if a.next_u64() != b.next_u64() {
                diferente = true;
                break;
            }
        }
        assert!(
            diferente,
            "semillas distintas deben producir secuencias distintas"
        );
    }

    #[test]
    fn test_rng_seed_zero_works() {
        let mut rng = Rng::new(0);
        let _ = rng.next_u64();
        let _ = rng.next_bounded(99);
    }

    #[test]
    fn test_rng_range_0_99() {
        let mut rng = Rng::new(12345);
        for _ in 0..10_000 {
            let v = rng.next_bounded(99);
            assert!((0..=99).contains(&v), "valor fuera de rango: {v}");
        }
    }

    #[test]
    fn test_rng_bounded_zero() {
        let mut rng = Rng::new(7);
        for _ in 0..100 {
            assert_eq!(rng.next_bounded(0), 0);
        }
    }

    #[test]
    fn test_shuffle_is_deterministic_permutation() {
        let mut a: Vec<i32> = (0..20).collect();
        let mut b: Vec<i32> = (0..20).collect();
        Rng::new(99).shuffle(&mut a);
        Rng::new(99).shuffle(&mut b);
        assert_eq!(a, b);

        let mut c: Vec<i32> = (0..20).collect();
        let mut d: Vec<i32> = (0..20).collect();
        Rng::new(100).shuffle(&mut c);
        Rng::new(100).shuffle(&mut d);
        assert_eq!(c, d);
        assert_ne!(
            a, c,
            "semillas distintas deberían producir órdenes distintos"
        );

        // Permutación: mismo contenido, distinto orden (salvo suerte)
        let mut orden: Vec<i32> = (0..20).collect();
        Rng::new(5).shuffle(&mut orden);
        let mut esperado: Vec<i32> = (0..20).collect();
        esperado.sort();
        let mut orden_sorted = orden.clone();
        orden_sorted.sort();
        assert_eq!(orden_sorted, esperado);
    }

    #[test]
    fn test_shuffle_empty_and_single() {
        let mut vacia: Vec<i32> = vec![];
        Rng::new(1).shuffle(&mut vacia);
        assert!(vacia.is_empty());

        let mut uno: Vec<i32> = vec![42];
        Rng::new(1).shuffle(&mut uno);
        assert_eq!(uno, vec![42]);
    }
}
