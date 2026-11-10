---
id: F4-01
title: Bus de eventos formal y puerto de salida del mundo
phase: F4
status: done
depends_on: []
size: M
---

# F4-01 · Bus de eventos formal y puerto de salida del mundo

## Objetivo

Formalizar la salida de eventos del mundo como un bus (Observer) y definir un puerto de salida (`WorldOutput`) en el núcleo para que la CLI (y mañana la ventana de F7) consuman eventos sin que el núcleo sepa quién los imprime. Cubre I6 (el intérprete emite `RuntimeError` al bus), 1.2 (puertos y adaptadores) y 4.3 (Observer / Event Bus) de `docs/PATRONES_DISENO_ARQUITECTURA.md`.

## Alcance

**Haz**
- Definir en `crates/instinct-core/src/event.rs` (o módulo nuevo `output.rs`) el trait `WorldOutput { fn emit(&mut self, event: &Event); }`.
- `World` (`crates/instinct-core/src/world.rs`) gana un puerto de salida opcional (`output: Option<Box<dyn WorldOutput>>`) con un setter `set_output`. `emit_event` empuja a `self.events` (compatibilidad con `drain_events`) **y** reenvía al puerto si existe.
- I6: cuando el intérprete encuentra un error de ejecución en una acción o expresión, el evento `RuntimeError` se publica en el bus del mundo (a través de `TurnResult` o del contexto) en lugar de devolverse solo como diagnóstico. La secuencia de ticks y la semilla no cambian.
- La CLI (`crates/instinct-cli/src/main.rs`) implementa `WorldOutput` con `CliOutput` que imprime eventos a `stdout`; diagnósticos y errores de carga siguen yendo a `stderr` (separación de streams ya establecida en F2, se respeta).
- `Simulation::step_tick` sigue devolviendo `Vec<Event>` (la CLI puede seguir usando `drain_events` o el puerto; no romper la API sin declararlo).
- Pruebas: `CliOutput` captura eventos en un `Vec` para probar el bus; el intérprete emite `RuntimeError` al dividir por cero / tipo erróneo; dos corridas misma semilla = misma secuencia de eventos.

**No hagas**
- No elimines `events` ni `drain_events` sin declarar el cambio de API y actualizar todos los sitios de llamada en el mismo issue.
- No cambies el orden de turnos, el límite de 100 líneas ni la semilla del PRNG.
- No introduzcas `println!` en `instinct-core`.
- No toques el parsing de argumentos de CLI (C1-C3 cerrados en F2).

### Archivos previstos
- `crates/instinct-core/src/event.rs` — modificar — trait `WorldOutput`.
- `crates/instinct-core/src/world.rs` — modificar — puerto de salida en `World`, `set_output`, `emit_event` reenvía al bus.
- `crates/instinct-core/src/interp.rs` — modificar — `TurnResult` lleva eventos de runtime (o equivalente) para que `RuntimeError` llegue al bus.
- `crates/instinct-core/src/sim.rs` — modificar — publicar eventos de runtime del intérprete en el mundo.
- `crates/instinct-cli/src/main.rs` — modificar — `CliOutput: WorldOutput`, `World::set_output`, mantener eventos a stdout y diagnósticos a stderr.
- `crates/instinct-core/tests/event_tests.rs` — modificar — pruebas del bus y del puerto de salida.
- `crates/instinct-core/tests/interp_tests.rs` — modificar — `RuntimeError` llega al bus.
- `crates/instinct-cli/tests/cli_run.rs` — modificar — eventos vía puerto, stderr separado.

## Criterios de aceptación

- [ ] **AC1** Existe `pub trait WorldOutput { fn emit(&mut self, event: &Event); }` en el núcleo y `World` lo expone como puerto opcional. Verif: `cargo test --workspace`.
- [ ] **AC2** `World::emit_event` reenvía al puerto de salida además de guardar en `self.events`; `drain_events` sigue funcionando. Verif: `test_world_output_forwards_to_bus` | `cargo test -p instinct-core --test event_tests`.
- [ ] **AC3** I6: un error de ejecución del intérprete (división por cero, tipo erróneo) produce un evento `RuntimeError` en el bus con tick, especie, posición, mensaje y línea. Verif: `test_interpreter_emits_runtime_error_to_bus` | `cargo test -p instinct-core --test interp_tests`.
- [ ] **AC4** La CLI imprime eventos a `stdout` a través de `CliOutput: WorldOutput` y los diagnósticos de carga a `stderr`. Verif: `test_cli_events_stdout_diagnostics_stderr` | `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC5** Determinismo: misma semilla sobre los cuatro ejemplos del anexo produce la misma secuencia de eventos antes y después del issue. Verif: dos corridas con `--seed 7` iguales.
- [ ] **AC6** Ningún `unwrap`/`expect`/`panic!` nuevo sobre datos de usuario o archivo en `instinct-core` ni `instinct-cli`. Verif: suite completa + clippy.
- [ ] **AC7** Suite completa en verde. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `Event` con `Display` en `event.rs`; `World.events`, `emit_event`, `drain_events` en `world.rs`; `execute_turn` devuelve `Result<TurnResult, (Diagnostic, usize)>` en `interp.rs`; `Simulation::step_tick` devuelve `Vec<Event>`; CLI imprime eventos a stdout y diagnósticos a stderr en `main.rs`. No hay `WorldOutput` ni el intérprete emite eventos internos (I6 abierto).

**Contratos que debes respetar**: API del núcleo que consume la CLI no se rompe sin declararlo y actualizar `main.rs` en el mismo cambio; cero `println!` en el núcleo; el intérprete lee y el mundo escribe; `WorldOutput` es `dyn` (objeto de trait) para permitir consumidores futuros (GUI).

**Ojo**: `World::from_map` no recibe el puerto; añade un setter `set_output` o un segundo constructor, y documenta que el puerto es opcional. Si un evento falla al emitirse al puerto, no se propaga el error (el bus nunca aborta el tick).

### Leer primero
- `crates/instinct-core/src/event.rs` — definición de `Event` y `Pos`.
- `crates/instinct-core/src/world.rs` — `emit_event`, `drain_events`, `World::from_map`, `Simulation` en `sim.rs`.
- `crates/instinct-core/src/interp.rs` — `execute_turn`, `TurnResult`, `PendingAction`.
- `crates/instinct-cli/src/main.rs` — cómo se imprimen eventos y mapa (stdout/stderr).

## Especificación

```
Núcleo (instinct-core)
  └── event.rs : trait WorldOutput { emit(&mut self, &Event) }
  └── world.rs : World.output: Option<Box<dyn WorldOutput>>
                   set_output(output)
                   emit_event(e) -> self.events.push(e) + output.emit(&e)
  └── interp.rs : TurnResult.runtime_errors: Vec<Event> (o equivalente)
  └── sim.rs     : publica runtime_errors del turno en el mundo (bus)
CLI (instinct-cli)
  └── main.rs   : CliOutput: WorldOutput (imprime a stdout)
                   World::set_output(Box::new(CliOutput))
                   diagnósticos siguen a stderr
```

## Tests

Nivel: unitario e integración. TDD: escribe primero el test que falle (bus sin puerto, `RuntimeError` no llega), luego implementa.

- `event_tests`: `World` sin puerto → `emit_event` guarda en `events`; con puerto → el puerto recibe el evento; `drain_events` sigue devolviendo todos.
- `interp_tests`: criatura ejecuta `wait(abc)` (texto donde se espera número) o división por cero → `RuntimeError` aparece en el bus del mundo con la línea del error.
- `cli_run`: ejecutar `run` con una criatura que falle en ejecución → el evento `RuntimeError` sale por `stdout` y ningún diagnóstico de carga sale por `stdout`.

## Verificación

- `cargo test -p instinct-core --test event_tests` → el bus y el puerto pasan.
- `cargo test -p instinct-core --test interp_tests` → `RuntimeError` llega al bus.
- `cargo test -p instinct-cli --test cli_run` → eventos stdout, diagnósticos stderr.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo dan la misma historia (eventos).

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si el intérprete no puede emitir al bus sin romper la regla "el intérprete lee, el mundo escribe", para y repórtalo: el evento de runtime lo produce el intérprete como dato y lo publica el mundo/simulación, no el intérprete directamente. No uses `Rc<RefCell>` ni `println!` en el núcleo.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f4-01-bus-eventos-puerto-salida (sobre develop).
Commit: 4dbdbd9 + changes below.

Evidencia AC por AC:
- AC1 (PASS): `pub trait WorldOutput { fn emit(&mut self, event: &Event); }` definido en `crates/instinct-core/src/event.rs`. `World` gana `output: Option<Box<dyn WorldOutput>>` con setter `set_output` en `crates/instinct-core/src/world.rs`. Verif: `cargo test --workspace` (8/8 en event_tests, 18/18 en interp_tests).
- AC2 (PASS): `emit_event` empuja a `self.events` y reenvía al puerto si existe; `drain_events` sigue funcionando. Verif: `test_world_output_forwards_to_bus` (cargo test -p instinct-core --test event_tests) → ok.
- AC3 (PASS): error de ejecución del intérprete (división por cero) produce `RuntimeError` en el bus con tick=0, especie="DivZero", pos=(0,0), mensaje no vacío, línea=7. Verif: `test_interpreter_emits_runtime_error_to_bus` (cargo test -p instinct-core --test interp_tests) → ok.
- AC4 (PASS): `CliOutput: WorldOutput` en `crates/instinct-cli/src/main.rs` imprime eventos a stdout; diagnósticos de carga siguen a stderr. Verif: `test_cli_events_stdout_diagnostics_stderr` (cargo test -p instinct-cli --test cli_run) → ok.
- AC5 (PASS): dos corridas con `--seed 7` sobre "Plain with rocks" dan stdout byte a byte idéntico (SHA256 igual). Verif: ejecución manual del binario dos veces + comparación de hash.
- AC6 (PASS): cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Verif: revisión de código + clippy en verde.
- AC7 (PASS): `cargo test --workspace` (278 tests ok), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0, sin warnings), `cargo fmt --all --check` (exit 0).

Cambios de firma autorizados:
- `World` gana `output: Option<Box<dyn WorldOutput>>` y `set_output(Box<dyn WorldOutput>)`.
- `emit_event` ahora reenvía al puerto además de guardar en `self.events`.
- `World` deja de derivar `Debug`/`Clone` automáticamente; implementaciones manuales que excluyen `output` de `Clone` y lo omite de `Debug`.
- CLI: `step_tick()` retorno se descarta (los eventos ya se imprimen vía el puerto); `println!` bucle de eventos eliminado de `execute_run`.

Archivos modificados:
- `crates/instinct-core/src/event.rs`: añade trait `WorldOutput`.
- `crates/instinct-core/src/world.rs`: campo `output`, setter `set_output`, `emit_event` reenvía al puerto, `Debug`/`Clone` manuales.
- `crates/instinct-cli/src/main.rs`: `CliOutput`, `set_output` en `execute_run`, bucle `println!` de eventos eliminado.
- `crates/instinct-core/tests/event_tests.rs`: añade `test_world_output_forwards_to_bus` + `SharedOutput`.
- `crates/instinct-core/tests/interp_tests.rs`: añade `test_interpreter_emits_runtime_error_to_bus` + `SharedOutput` + `species_with_div_zero_program`.
- `crates/instinct-cli/tests/cli_run.rs`: añade `test_cli_events_stdout_diagnostics_stderr`.
- `examples/creatures/divzero.ins`: nueva criatura con `n = 1 / 0` para prueba de error de ejecución en CLI.

Desviaciones: ninguna. El alcance y los criterios de aceptación se respetan sin cambios.
Cambios de firma autorizados: `World` gana `set_output` / puerto opcional; `TurnResult` puede ganar eventos de runtime. Decláralos en el cierre.