//! Own seeded pseudo-random number generator.
//!
//! Algorithm: SplitMix64. A simple, fast, deterministic 64-bit permutation,
//! independent of platform and Rust version (no `usize` in any generation
//! operation). Each call to `next_u64` advances the internal state and
//! returns a new value.

/// Source of pseudo-randomness. The `Strategy` pattern lets tests inject a
/// deterministic `TestRng` while production uses `Rng` (SplitMix64).
pub trait RngSource: std::fmt::Debug {
    /// Returns an integer between 0 and `max` inclusive (maximum included).
    /// `max` must be >= 0; if negative it is treated as 0.
    fn next_bounded(&mut self, max: i64) -> i64;

    /// Shuffles `list` (a slice of indices) in-place using Fisher–Yates with
    /// this same generator. Preserves all elements and with the same seed
    /// produces the same order.
    fn shuffle(&mut self, list: &mut [usize]);
}

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

    /// Shuffles `list` in-place using Fisher–Yates with this same generator.
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

impl RngSource for Rng {
    fn next_bounded(&mut self, max: i64) -> i64 {
        Rng::next_bounded(self, max)
    }

    fn shuffle(&mut self, list: &mut [usize]) {
        let mut i = list.len();
        while i > 1 {
            i -= 1;
            let j = self.next_bounded(i as i64) as usize;
            list.swap(i, j);
        }
    }
}
