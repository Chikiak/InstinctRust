use instinct_core::rng::RngSource;

/// Deterministic RNG for tests: returns values from a fixed sequence.
#[derive(Debug, Clone)]
pub struct TestRng {
    values: Vec<i64>,
    index: usize,
}

impl TestRng {
    pub fn new(values: Vec<i64>) -> Self {
        TestRng { values, index: 0 }
    }
}

impl RngSource for TestRng {
    fn next_bounded(&mut self, max: i64) -> i64 {
        let v = self.values[self.index % self.values.len()];
        self.index += 1;
        if max < 0 {
            return 0;
        }
        let range = (max as u64).wrapping_add(1);
        (v as u64 % range) as i64
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
