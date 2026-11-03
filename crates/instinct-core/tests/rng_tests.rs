//! Unit tests for the `rng` module.

use instinct_core::rng::Rng;

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
