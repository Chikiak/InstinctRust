//! Events emitted by the world during simulation.
//! The world never prints; it only emits events that consumers (CLI, GUI)
//! render.

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
    /// Una criatura rugió. `tick` es el tick en que se emitió el rugido; el
    /// mensaje queda disponible para las criaturas cercanas el tick siguiente
    /// (F2 lo lee con `roars_near`/`last_roar` filtrando por `tick_actual - 1`).
    Roared {
        tick: u64,
        species: String,
        pos: Pos,
        text: String,
    },
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Said {
                tick,
                species,
                pos,
                text,
            } => {
                write!(
                    f,
                    "[tick {tick}] {species}@{x},{y} dice: {text}",
                    x = pos.x,
                    y = pos.y
                )
            }
            Event::Died {
                tick,
                species,
                pos,
                cause,
            } => {
                write!(
                    f,
                    "[tick {tick}] {species}@{x},{y} muere por {cause}",
                    x = pos.x,
                    y = pos.y
                )
            }
            Event::RuntimeError {
                tick,
                species,
                pos,
                message,
                line,
            } => {
                write!(
                    f,
                    "[tick {tick}] {species}@{x},{y} error en línea {line}: {message}",
                    x = pos.x,
                    y = pos.y
                )
            }
            Event::Damaged {
                tick,
                species,
                pos,
                target,
                amount,
            } => {
                write!(
                    f,
                    "[tick {tick}] {species}@{x},{y} daña {tx},{ty} por {amount}",
                    x = pos.x,
                    y = pos.y,
                    tx = target.x,
                    ty = target.y
                )
            }
            Event::Roared {
                tick,
                species,
                pos,
                text,
            } => {
                write!(
                    f,
                    "[tick {tick}] {species}@{x},{y} ruge: {text}",
                    x = pos.x,
                    y = pos.y
                )
            }
        }
    }
}
