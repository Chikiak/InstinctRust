//! Own seeded pseudo-random number generator.
//!
//! Algorithm: SplitMix64. A simple, fast, deterministic 64-bit permutation,
//! independent of platform and Rust version (no `usize` in any generation
//! operation). Each call to `next_u64` advances the internal state and
//! returns a new value.

/// Seeded pseudo-random number generator with a 64-bit seed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rng {
    state: u64,
}

const SPLITMIX64_INCREMENT: u64 = 0x9E3779B97F4A7C15;
const SPLITMIX64_MUL1: u64 = 0xBF58476D1CE4E5B9;
const SPLITMIX64_MUL2: u64 = 0x94D049BB133111EB;

impl Rng {
    /// Builds a generator from a 64-bit seed (seed 0 works).
    pub fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    /// Advances the state and returns the next u64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX64_INCREMENT);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX64_MUL1);
        z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX64_MUL2);
        z ^ (z >> 31)
    }

    /// Returns an integer between 0 and `max` inclusive (maximum included).
    /// `max` must be >= 0; if negative it is treated as 0.
    pub fn next_bounded(&mut self, max: i64) -> i64 {
        let max = if max < 0 { 0 } else { max };
        let range = (max as u64).wrapping_add(1);
        (self.next_u64() % range) as i64
    }

    /// Shuffles `lista` in-place using Fisher–Yates with this same generator.
    /// Preserves all elements and with the same seed produces the same order.
    pub fn shuffle<T>(&mut self, list: &mut [T]) {
        let mut i = list.len();
        while i > 1 {
            i -= 1;
            let j = self.next_bounded(i as i64) as usize;
            list.swap(i, j);
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
        let mut different = false;
        for _ in 0..100 {
            if a.next_u64() != b.next_u64() {
                different = true;
                break;
            }
        }
        assert!(
            different,
            "different seeds must produce different sequences"
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
            assert!((0..=99).contains(&v), "value out of range: {v}");
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
        assert_ne!(a, c, "different seeds should produce different orders");

        // Permutation: same content, different order (unless by chance)
        let mut order: Vec<i32> = (0..20).collect();
        Rng::new(5).shuffle(&mut order);
        let mut expected: Vec<i32> = (0..20).collect();
        expected.sort();
        let mut order_sorted = order.clone();
        order_sorted.sort();
        assert_eq!(order_sorted, expected);
    }

    #[test]
    fn test_shuffle_empty_and_single() {
        let mut empty: Vec<i32> = vec![];
        Rng::new(1).shuffle(&mut empty);
        assert!(empty.is_empty());

        let mut one: Vec<i32> = vec![42];
        Rng::new(1).shuffle(&mut one);
        assert_eq!(one, vec![42]);
    }
}
