//! Runtime values: 64-bit integers or text, with the evaluation semantics
//! of the Instinct language.
//!
//! Integer division rounds toward minus infinity (floor division) and the
//! remainder takes the sign of the divisor, matching the enunciado. All
//! arithmetic is checked: overflow, division by zero and modulo by zero are
//! runtime errors.

pub mod cmp;
pub mod div_rem;
pub mod ops;
pub mod truth;

/// A value in the Instinct language: a signed 64-bit integer or text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Text(String),
}
