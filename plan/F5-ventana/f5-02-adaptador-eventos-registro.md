---
id: F5-02
title: Adaptador WorldOutput para GUI + panel de registro
status: done
phase: F5-ventana
depends_on: [F5-01]
size: M
file: f5-02-adaptador-eventos-registro.md
---

# F5-02 · Adaptador de eventos y panel de registro (headless)

## Leer primero

- `crates/instinct-core/src/event/output.rs` (trait `WorldOutput`, L10-13) y
  `event/events.rs` (variantes de `Event`, L13-57)
- `crates/instinct-core/src/world/ticks.rs` (`set_output` L27, `drain_events` L54)
- `crates/instinct-cli/src/output.rs` (`CliOutput`, el adaptador de consola)
- `plan/F5-ventana/PHASE.md` (la GUI es un adaptador; lógica headless testeable)

## Tarea

1. Implementar en `instinct-gui` un adaptador del puerto: `struct LogBuffer` (o
   equivalente) que implemente `WorldOutput` y **encole** cada `Event` como una línea
   de registro legible (`[tick N] Species@x,y says: ...`), sin imprimir nada. El
   formateo de cada variante de evento se hace con un `match` exhaustivo sobre `Event`
   (nunca `if let` encadenados):
   - `Said`, `Died`, `RuntimeError` (¡con archivo/línea del mensaje!), `Damaged`,
     `Roared` quedan distinguibles por tipo de evento en el registro.
2. El registro debe aceptar también **diagnósticos de carga** (`Diagnostic` de
   `resources::load_resources`, `resources/mod.rs:47`) como entradas con marca de
   "load", ya que 4.1 exige errores de carga al arrancar.
3. API del registro: `push_event(&Event)`, `push_diagnostic(&Diagnostic)`,
   `entries() -> &[LogEntry]`, `clear()`. Estructura de datos acotada (máximo de
   entradas configurable, descartando las más antiguas) para que una simulación larga
   no crezca sin límite.
4. **Cero acoplamiento con eframe** en este módulo: todo en `app/log.rs` (o
   equivalente) y cubierto por tests de integración; la capa `ui/` en F5-06 lo pinta.
5. Conectar en `main.rs` (modo spike) el `LogBuffer` vía `sim.world.set_output(...)`
   con un mundo mínimo construido en test/binario para demostrar que los eventos de
   `step_tick` llegan al buffer.

## Criterios de aceptación

1. Existe `impl WorldOutput for LogBuffer` y se conecta con `set_output` sin tocar el
   núcleo (git sin cambios en `crates/instinct-core/`).
2. Tests (sin ventana): las cinco variantes de `Event` producen entradas de registro
   correctos; un `Diagnostic` de carga entra como entrada "load"; el límite de
   entradas descarta las antiguas; `clear` vacía.
3. Los eventos de una simulación real (`Simulation::step_tick` sobre `examples`) se
   acumulan en el buffer por el puerto, no por leer el retorno de `step_tick`.
4. `cargo test --workspace` exit 0; clippy `-D warnings` exit 0; `fmt --check` exit 0.
5. Ningún archivo > ~300 líneas salvo excepción anotada; `git status` limpio.

## Verificación

```powershell
cargo test -p instinct-gui
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

## Notas de implementación

Branch: `feat/f5-02-adaptador-eventos-registro` (no se cambió ni se creó rama; commit sobre la rama actual).

AC-by-AC evidence:
- AC1 (PASS): `impl WorldOutput for LogBuffer` en `crates/instinct-gui/src/app/log.rs`, conectado con `sim.world.set_output(...)` en `tests/log/simulation.rs` y `src/main.rs`; `git diff develop -- crates/instinct-core/` vacío (0 cambios en el núcleo).
- AC2 (PASS): `cargo test -p instinct-gui --test log` → 8/8 ok: las cinco variantes de `Event` (`Said`, `Died`, `RuntimeError`, `Damaged`, `Roared`) con `match` exhaustivo; `Diagnostic` como `load file:line: message`; límite descarta las antiguas (FIFO); `clear` vacía.
- AC3 (PASS): `simulation::step_tick_events_accumulate_in_buffer_via_port`: `Simulation::step_tick` sobre `examples` con `set_output(ProbeBuffer)` (`Rc<RefCell<LogBuffer>>` + `WorldOutput` delegado); retorno de `step_tick` ignorado (`let _ =`); entradas inspeccionadas por el puerto, no por el valor de retorno.
- AC4 (PASS): `cargo test --workspace` → exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all --check` → exit 0.
- AC5 (PASS): `src/app/log.rs` sin eframe/egui; ningún archivo > ~300 líneas; `git status --short` limpio tras el commit.

Signature changes: ninguna en `instinct-core`. API nueva pública: `instinct_gui::{LogBuffer, LogEntry}` y `LogBuffer::{with_capacity, capacity, set_capacity, push_event, push_diagnostic, entries, clear}` + `impl WorldOutput for LogBuffer`.

Modified files:
- `crates/instinct-gui/Cargo.toml` (nuevo `[[test]] name = "log"`)
- `crates/instinct-gui/src/app/mod.rs` (`pub mod log;`)
- `crates/instinct-gui/src/lib.rs` (`pub use app::log::{LogBuffer, LogEntry};`)
- `crates/instinct-gui/src/main.rs` (modo `--smoke` real: mundo mínimo de `examples`, `LogBuffer` vía `set_output`, `step_tick` N veces sin ventana)
- `crates/instinct-gui/src/app/log.rs` (nuevo: `LogBuffer`, `LogEntry`, `LogEntryKind`, formateo por `match` exhaustivo, descarte FIFO)
- `crates/instinct-gui/tests/log/mod.rs`, `tests/log/buffer.rs`, `tests/log/simulation.rs` (nuevos)

Deviations: ninguna.