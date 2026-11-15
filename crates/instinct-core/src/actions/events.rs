//! Damage event construction.
use crate::event::{Event, Pos};

pub(super) fn damaged_event(tick: u64, species: &str, pos: Pos, target: Pos, amount: i64) -> Event {
    Event::Damaged {
        tick,
        species: species.to_string(),
        pos,
        target,
        amount,
    }
}
