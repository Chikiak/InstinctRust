//! Display formatting for world events.

use crate::event::events::Event;

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
                    "[tick {tick}] {species}@{x},{y} says: {text}",
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
                    "[tick {tick}] {species}@{x},{y} dies of {cause}",
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
                    "[tick {tick}] {species}@{x},{y} error at line {line}: {message}",
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
                    "[tick {tick}] {species}@{x},{y} damages {tx},{ty} by {amount}",
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
                    "[tick {tick}] {species}@{x},{y} roars: {text}",
                    x = pos.x,
                    y = pos.y
                )
            }
        }
    }
}
