//! A position on the map grid and the events produced by creature actions
//! and by the laws of the world.

/// A position on the map grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

/// Events produced by creature actions and by the laws of the world.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A creature said something. `tick` is the current tick, `species` the
    /// creature's species name, `pos` its position at that moment, and `text`
    /// the spoken text (a number is shown as text).
    Said {
        tick: u64,
        species: String,
        pos: Pos,
        text: String,
    },
    /// A creature died. `cause` is `vida`, `edad`, or `error`.
    Died {
        tick: u64,
        species: String,
        pos: Pos,
        cause: String,
    },
    /// A runtime error killed a creature.
    RuntimeError {
        tick: u64,
        species: String,
        pos: Pos,
        message: String,
        line: usize,
    },
    /// Damage applied to a cell's content (creature health, object reserve, or
    /// terrain reserve). `pos` is the attacker's position, `target` the cell
    /// that was hit, and `amount` the damage dealt (negative = healing).
    Damaged {
        tick: u64,
        species: String,
        pos: Pos,
        target: Pos,
        amount: i64,
    },
    /// A creature roared. `tick` is the tick when the roar was emitted; the
    /// message remains available to nearby creatures the following tick
    /// (F2 reads it with `roars_near`/`last_roar` filtering by
    /// `tick_actual - 1`).
    Roared {
        tick: u64,
        species: String,
        pos: Pos,
        text: String,
    },
}
