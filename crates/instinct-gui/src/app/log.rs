//! Headless log buffer: a `WorldOutput` adapter that stores every world
//! event (and every load diagnostic) as a bounded, human-readable line.
//! No knowledge of eframe/egui: pure Rust, testable without opening a window.

use instinct_core::diagnostic::Diagnostic;
use instinct_core::event::{Event, WorldOutput};

/// Kind of log entry: a world event or a load diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogEntryKind {
    #[default]
    Event,
    Load,
}

/// One line of the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Tick of the event (`0` for load diagnostics, which carry no tick).
    pub tick: u64,
    /// `Event` for world events, `Load` for load diagnostics.
    pub kind: LogEntryKind,
    /// Rendered line, e.g. `[tick 3] Dwarf@1,0 says: hello`.
    pub line: String,
}

/// Bounded FIFO log of entries. Implements `WorldOutput` so the world bus
/// delivers events directly to this buffer. When full, the oldest entries
/// are discarded so a long simulation never grows without bound.
pub struct LogBuffer {
    entries: Vec<LogEntry>,
    capacity: usize,
}

impl LogBuffer {
    /// Creates a buffer with the given maximum number of entries.
    pub fn with_capacity(capacity: usize) -> Self {
        LogBuffer {
            entries: Vec::with_capacity(capacity),
            capacity,
        }
    }

    /// Returns the maximum number of entries kept.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Changes capacity, discarding the oldest entries if needed.
    pub fn set_capacity(&mut self, capacity: usize) {
        while self.entries.len() > capacity {
            self.entries.remove(0);
        }
        self.capacity = capacity;
    }

    /// Pushes a world event, formatted with an exhaustive `match` over `Event`.
    pub fn push_event(&mut self, event: &Event) {
        let (tick, line) = match event {
            Event::Said {
                tick,
                species,
                pos,
                text,
            } => (
                *tick,
                format!(
                    "[tick {tick}] {species}@{x},{y} says: {text}",
                    x = pos.x,
                    y = pos.y
                ),
            ),
            Event::Died {
                tick,
                species,
                pos,
                cause,
            } => (
                *tick,
                format!(
                    "[tick {tick}] {species}@{x},{y} dies of {cause}",
                    x = pos.x,
                    y = pos.y
                ),
            ),
            Event::RuntimeError {
                tick,
                species,
                pos,
                message,
                line,
            } => (
                *tick,
                format!(
                    "[tick {tick}] {species}@{x},{y} error at line {line}: {message}",
                    x = pos.x,
                    y = pos.y
                ),
            ),
            Event::Damaged {
                tick,
                species,
                pos,
                target,
                amount,
            } => (
                *tick,
                format!(
                    "[tick {tick}] {species}@{x},{y} damages {tx},{ty} by {amount}",
                    x = pos.x,
                    y = pos.y,
                    tx = target.x,
                    ty = target.y
                ),
            ),
            Event::Roared {
                tick,
                species,
                pos,
                text,
            } => (
                *tick,
                format!(
                    "[tick {tick}] {species}@{x},{y} roars: {text}",
                    x = pos.x,
                    y = pos.y
                ),
            ),
        };
        self.push(LogEntry {
            tick,
            kind: LogEntryKind::Event,
            line,
        });
    }

    /// Pushes a load diagnostic as `load file:line: message`.
    pub fn push_diagnostic(&mut self, diagnostic: &Diagnostic) {
        self.push(LogEntry {
            tick: 0,
            kind: LogEntryKind::Load,
            line: format!(
                "load {}:{}: {}",
                diagnostic.file, diagnostic.line, diagnostic.message
            ),
        });
    }

    /// Returns all entries, oldest first.
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Removes all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.capacity == 0 {
            return;
        }
        if self.entries.len() == self.capacity {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        LogBuffer::with_capacity(1024)
    }
}

impl WorldOutput for LogBuffer {
    fn emit(&mut self, event: &Event) {
        self.push_event(event);
    }
}
