use instinct_core::event::WorldOutput;

/// World output port for the CLI: prints every event to stdout.
/// Load diagnostics still go to stderr through separate paths.
pub(crate) struct CliOutput;

impl WorldOutput for CliOutput {
    fn emit(&mut self, event: &instinct_core::event::Event) {
        println!("{event}");
    }
}
