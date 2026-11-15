use std::path::Path;
use std::rc::Rc;

use instinct_core::resources;
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::world;

use crate::args::{parse_place, Placement, RunArgs};
use crate::output::CliOutput;

/// Executes the `run` subcommand. Returns the exit code.
/// The map and tick events go to stdout; diagnostics go to stderr (C5).
pub(crate) fn execute_run(args: &RunArgs) -> i32 {
    let root = Path::new(&args.root);
    let resources = resources::load_resources(root);

    // Load diagnostics are reported to stderr.
    for d in &resources.diagnostics {
        eprintln!("{d}");
    }

    // Find the map by name.
    let map = resources.maps.iter().find(|m| m.name == args.map_name);
    let map = match map {
        Some(m) => m,
        None => {
            eprintln!("map not found: '{}'", args.map_name);
            if !resources.maps.is_empty() {
                eprint!("mapas disponibles: ");
                for (i, m) in resources.maps.iter().enumerate() {
                    if i > 0 {
                        eprint!(", ");
                    }
                    eprint!("{}", m.name);
                }
                eprintln!();
            }
            return 1;
        }
    };

    if resources.terrains.is_empty() {
        eprintln!("no terrains loaded to draw the map");
        return 1;
    }

    // Parse every --place value. A malformed placement or an unknown species
    // aborts with exit code 1 before simulating.
    let mut placements: Vec<(Placement, Rc<Species>)> = Vec::new();
    for place_str in &args.places {
        let placement = match parse_place(place_str) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{e}");
                return 1;
            }
        };
        let species = match resources
            .creatures
            .iter()
            .find(|s| s.name == placement.species)
        {
            Some(s) => Rc::new(s.clone()),
            None => {
                eprintln!("unknown species: '{}'", placement.species);
                return 1;
            }
        };
        placements.push((placement, species));
    }

    // Build the world and place creatures. Each creature points to the same
    // `Rc<Species>` that the world registers in its catalog (flyweight).
    let mut world = world::World::from_map(map, &resources.terrains, &resources.objects);
    for d in &world.diagnostics {
        eprintln!("{d}");
    }

    for (placement, species) in &placements {
        match world.place_creature(species, placement.x, placement.y) {
            Ok(_) => {}
            Err(d) => {
                eprintln!("{d}");
                return 1;
            }
        }
    }

    // Create the simulation and seed its generator. The world is now the sole
    // owner of the terrain and object catalogs.
    let mut sim = Simulation::new(world);
    // Connect the output port: events go to stdout via CliOutput.
    sim.world.set_output(Box::new(CliOutput));
    sim.set_rng_seed(args.seed);

    let ticks = args.ticks;
    let every = args.every;

    // Map at tick 0 (initial state).
    println!("== tick 0 ==");
    match sim.world().draw() {
        Ok(drawing) => print!("{drawing}"),
        Err(d) => {
            eprintln!("{d}");
            return 1;
        }
    }

    for _ in 0..ticks {
        // step_tick advances the tick and events are printed through the
        // output port (CliOutput). The return value is discarded; the API
        // still returns Vec<Event> for compatibility.
        let _ = sim.step_tick();

        let next_tick = sim.tick();

        // Print the map when the tick counter reaches a multiple of `--every`
        // or the final tick.
        let should_print =
            every > 0 && (next_tick.is_multiple_of(every as u64) || next_tick == ticks as u64);
        if should_print {
            println!("== tick {next_tick} ==");
            match sim.world().draw() {
                Ok(drawing) => print!("{drawing}"),
                Err(d) => {
                    eprintln!("{d}");
                    return 1;
                }
            }
        }
    }

    0
}
