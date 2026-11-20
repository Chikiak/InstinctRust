//! AC3: a real simulation (`Simulation::step_tick` over `examples`) accumulates
//! its events in the buffer through the `WorldOutput` port, not by reading the
//! return value of `step_tick`.

use std::cell::RefCell;
use std::rc::Rc;

use instinct_core::event::WorldOutput;
use instinct_core::resources;
use instinct_core::sim::Simulation;
use instinct_core::world;

use instinct_gui::LogBuffer;

/// Mirror of the buffer behind the port, for inspection after the simulation.
struct ProbeBuffer(Rc<RefCell<LogBuffer>>);

impl WorldOutput for ProbeBuffer {
    fn emit(&mut self, event: &instinct_core::event::Event) {
        self.0.borrow_mut().push_event(event);
    }
}

/// Root of `examples`, resolved from this crate's manifest dir so tests work
/// regardless of the current directory (tests run with cwd = crate root).
fn examples_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

/// Minimal world from `examples`: map "Plain with rocks" with a `Talker`
/// (says "hola" on its first turn at tick 0).
fn world_with_talker() -> world::World {
    let resources = resources::load_resources(&examples_root());
    let map = resources
        .maps
        .iter()
        .find(|m| m.name == "Plain with rocks")
        .expect("examples must contain the map 'Plain with rocks'");
    let species = resources
        .creatures
        .iter()
        .find(|s| s.name == "Talker")
        .expect("examples must contain the species 'Talker'")
        .clone();

    let mut world = world::World::from_map(map, &resources.terrains, &resources.objects);
    let species = Rc::new(species);
    world
        .place_creature(&species, 1, 0)
        .expect("placing Talker at (1, 0) on a free cell");
    world
}

#[test]
fn step_tick_events_accumulate_in_buffer_via_port() {
    let buffer = Rc::new(RefCell::new(LogBuffer::with_capacity(100)));
    let port = ProbeBuffer(Rc::clone(&buffer));

    let mut sim = Simulation::new(world_with_talker());
    sim.world.set_output(Box::new(port));
    sim.set_rng_seed(7);

    // The return value of `step_tick` is deliberately ignored: events must
    // arrive exclusively through the output port.
    for _ in 0..3 {
        let _ = sim.step_tick();
    }

    let borrow = buffer.borrow();
    let entries = borrow.entries();
    assert!(
        !entries.is_empty(),
        "no events arrived through the port; check set_output wiring"
    );
    assert!(
        entries.iter().any(|e| e.line.contains("says:")),
        "expected a Said event from the Talker, got: {entries:?}"
    );
    assert!(
        entries.iter().all(|e| e.line.starts_with("[tick")),
        "every log line must be tick-prefixed, got: {entries:?}"
    );
}
