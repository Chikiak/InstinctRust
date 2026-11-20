//! `LogBuffer` in isolation: event variants, load diagnostics,
//! bounded FIFO capacity, and `clear`.

use instinct_core::diagnostic::Diagnostic;
use instinct_core::event::{Event, Pos};
use instinct_gui::app::log::{LogBuffer, LogEntryKind};

fn said_at(tick: u64) -> Event {
    Event::Said {
        tick,
        species: "Dwarf".to_string(),
        pos: Pos { x: 1, y: 0 },
        text: "hello".to_string(),
    }
}

fn died_at(tick: u64) -> Event {
    Event::Died {
        tick,
        species: "Dwarf".to_string(),
        pos: Pos { x: 1, y: 0 },
        cause: "health".to_string(),
    }
}

fn runtime_error_at(tick: u64) -> Event {
    Event::RuntimeError {
        tick,
        species: "Dwarf".to_string(),
        pos: Pos { x: 1, y: 0 },
        message: "division by zero".to_string(),
        line: 42,
    }
}

fn damaged_at(tick: u64) -> Event {
    Event::Damaged {
        tick,
        species: "Dwarf".to_string(),
        pos: Pos { x: 1, y: 0 },
        target: Pos { x: 3, y: 1 },
        amount: 10,
    }
}

fn roared_at(tick: u64) -> Event {
    Event::Roared {
        tick,
        species: "Dwarf".to_string(),
        pos: Pos { x: 1, y: 0 },
        text: "khazad ai-menu".to_string(),
    }
}

#[test]
fn said_entry_matches_required_format() {
    let mut log = LogBuffer::with_capacity(10);
    log.push_event(&said_at(3));

    let entries = log.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tick, 3);
    assert_eq!(entries[0].kind, LogEntryKind::Event);
    assert_eq!(entries[0].line, "[tick 3] Dwarf@1,0 says: hello");
}

#[test]
fn five_event_variants_produce_distinguishable_entries() {
    let mut log = LogBuffer::with_capacity(10);
    for event in [
        said_at(1),
        died_at(2),
        runtime_error_at(3),
        damaged_at(4),
        roared_at(5),
    ] {
        log.push_event(&event);
    }

    let entries = log.entries();
    assert_eq!(entries.len(), 5);
    assert!(entries.iter().all(|e| e.kind == LogEntryKind::Event));

    let lines: Vec<&str> = entries.iter().map(|e| e.line.as_str()).collect();
    assert_eq!(lines[0], "[tick 1] Dwarf@1,0 says: hello");
    assert_eq!(lines[1], "[tick 2] Dwarf@1,0 dies of health");
    assert!(
        lines[2].contains("error at line 42"),
        "runtime error: {lines:?}"
    );
    assert_eq!(
        lines[2],
        "[tick 3] Dwarf@1,0 error at line 42: division by zero"
    );
    assert_eq!(lines[3], "[tick 4] Dwarf@1,0 damages 3,1 by 10");
    assert_eq!(lines[5 - 1], "[tick 5] Dwarf@1,0 roars: khazad ai-menu");
}

#[test]
fn diagnostic_enters_as_load_entry() {
    let mut log = LogBuffer::with_capacity(10);
    log.push_diagnostic(&Diagnostic::new(
        "resources/terrain/grass.te",
        7,
        "unknown terrain",
    ));

    let entries = log.entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].tick, 0);
    assert_eq!(entries[0].kind, LogEntryKind::Load);
    assert_eq!(
        entries[0].line,
        "load resources/terrain/grass.te:7: unknown terrain"
    );
}

#[test]
fn capacity_discards_oldest_entries_fifo() {
    let mut log = LogBuffer::with_capacity(3);
    for tick in 1..=5 {
        log.push_event(&said_at(tick));
    }

    let entries = log.entries();
    assert_eq!(entries.len(), 3);
    let ticks: Vec<u64> = entries.iter().map(|e| e.tick).collect();
    assert_eq!(
        ticks,
        vec![3, 4, 5],
        "oldest entries must be discarded first"
    );
}

#[test]
fn set_capacity_shrinks_and_discards_oldest() {
    let mut log = LogBuffer::with_capacity(5);
    for tick in 1..=5 {
        log.push_event(&said_at(tick));
    }
    log.set_capacity(2);

    let entries = log.entries();
    assert_eq!(entries.len(), 2);
    let ticks: Vec<u64> = entries.iter().map(|e| e.tick).collect();
    assert_eq!(ticks, vec![4, 5]);
}

#[test]
fn clear_empties_the_buffer() {
    let mut log = LogBuffer::with_capacity(10);
    log.push_event(&said_at(1));
    log.push_diagnostic(&Diagnostic::new("file", 1, "msg"));
    assert_eq!(log.entries().len(), 2);

    log.clear();
    assert!(log.entries().is_empty());

    // The buffer keeps working after clearing.
    log.push_event(&said_at(2));
    assert_eq!(log.entries().len(), 1);
    assert_eq!(log.entries()[0].line, "[tick 2] Dwarf@1,0 says: hello");
}

#[test]
fn zero_capacity_keeps_nothing() {
    let mut log = LogBuffer::with_capacity(0);
    log.push_event(&said_at(1));
    assert!(log.entries().is_empty());
}
