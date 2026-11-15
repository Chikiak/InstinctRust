use super::common::*;
use instinct_core::rng::{Rng, RngSource};

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

#[test]
fn test_rng_source_trait_is_implemented_by_rng() {
    // AC1: Rng implements RngSource and the trait methods work.
    let mut rng: Box<dyn RngSource> = Box::new(Rng::new(42));
    let v1 = rng.next_bounded(99);
    let v2 = rng.next_bounded(99);
    assert!((0..=99).contains(&v1), "value out of range: {v1}");
    assert!((0..=99).contains(&v2), "value out of range: {v2}");
    assert_ne!(v1, v2, "consecutive draws differ");

    let mut list: Vec<usize> = (0..10).collect();
    rng.shuffle(&mut list);
    assert_eq!(list.len(), 10);
    let mut sorted = list.clone();
    sorted.sort();
    assert_eq!(sorted, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
}

#[test]
fn test_test_rng_next_bounded_returns_fixed_sequence() {
    // TestRng returns values from its fixed sequence, bounded by max.
    let mut rng = TestRng::new(vec![1, 2, 3]);
    assert_eq!(rng.next_bounded(99), 1); // 1 % 100 = 1
    assert_eq!(rng.next_bounded(99), 2); // 2 % 100 = 2
    assert_eq!(rng.next_bounded(99), 3); // 3 % 100 = 3
    assert_eq!(rng.next_bounded(99), 1); // wraps to 1 % 100 = 1
}

#[test]
fn test_test_rng_shuffle_is_deterministic() {
    // TestRng shuffle uses next_bounded and is deterministic per sequence.
    let mut a: Vec<usize> = (0..6).collect();
    let mut b: Vec<usize> = (0..6).collect();
    TestRng::new(vec![0, 0, 0, 0, 0, 0]).shuffle(&mut a);
    TestRng::new(vec![0, 0, 0, 0, 0, 0]).shuffle(&mut b);
    assert_eq!(a, b, "same TestRng sequence -> same shuffle");

    // All zeros: next_bounded(i) = 0 % (i+1) = 0, j=0 always ->
    // Fisher-Yates rotates: [1, 2, 3, 4, 5, 0].
    let mut rev: Vec<usize> = (0..6).collect();
    TestRng::new(vec![0]).shuffle(&mut rev);
    assert_eq!(rev, vec![1, 2, 3, 4, 5, 0]);
}
