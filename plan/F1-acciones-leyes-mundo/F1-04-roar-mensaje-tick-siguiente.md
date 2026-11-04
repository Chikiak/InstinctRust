---
id: F1-04
title: roar: mensaje disponible el tick siguiente
phase: F1
status: done
depends_on: [F1-01]
size: M
---

# F1-04 · roar: mensaje disponible el tick siguiente

## Objetivo
Implementar la acción `roar` que emite un mensaje de rugido; las criaturas que lo oigan pueden leer su texto en el tick siguiente (la lectura de rugidos desde el lenguaje, `roars_near`/`last_roar`, es F2). En F1 se almacena el rugido y se emite el evento `Roared`.

## Alcance
**Haz**
- Añadir el tipo `Roar` que implemente `Action` en `crates/instinct-core/src/actions.rs`.
- Registrar `roar` en `Catalog::default()` con aridad 1.
- `roar` consume turno aunque el mensaje sea vacío o un número (se muestra como texto).
- Cualquier valor (entero o texto) es válido; un entero se muestra como su representación textual.
- Almacenar el rugido como "rugido del tick anterior" asociado al tick y posición de quien ruge, para que F2 lo lea con `roars_near`/`last_roar` dentro del radio de visión.
- Emitir un evento `Roared` con tick, especie, posición y texto.
- Decisión de PHASE.md: no hay evento independiente de "disponible el tick siguiente"; el almacenamiento del rugido con su tick es suficiente para que F2 filtre por `tick_actual - 1`.
- Tests unitarios que cubran emisión de evento, texto numérico convertido a texto, y que el rugido se almacene con su tick.
- `cargo test -p instinct-core`, suite completa, clippy y fmt en verde.

**No hagas**
- No implementes `roars_near`, `last_roar` ni `see`/`name` (F2).
- No hagas que el rugido dañe, cure ni afecte al mundo más allá del evento y el almacenamiento.
- No cambies la semántica de `say` (sigue sin consumir turno).

### Archivos previstos
- `crates/instinct-core/src/actions.rs` — modificar — añadir `Roar` y registrarlo.
- `crates/instinct-core/src/event.rs` — modificar — añadir variante `Roared` al enum `Event` y su `Display`.
- `crates/instinct-core/src/world.rs` — modificar — almacén de rugidos pendientes (tick, especie, posición, texto) con método para añadir y para consultar los del tick anterior (preparando F2).
- `crates/instinct-core/src/sim.rs` — modificar — al final de cada tick, rotar los rugidos: los del tick actual pasan a ser "del tick anterior" para el siguiente.
- `crates/instinct-core/tests/actions_tests.rs` — modificar — añadir tests de `roar`.

## Criterios de aceptación
- [ ] **AC1** `roar("texto")` emite un evento `Roared` con tick, especie, posición y texto del rugido. Verif: `test_roar_emits_event` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC2** `roar(42)` con número muestra "42" como texto. Verif: `test_roar_number_as_text` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC3** `roar` consume turno (`consumes_turn() == true`). Verif: `test_roar_consumes_turn` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC4** El rugido se almacena asociado al tick en que se emitió. Verif: `test_roar_stored_with_tick` | `cargo test -p instinct-core --test actions_tests` o `sim_tests` | prueba pasa.
- [ ] **AC5** `Catalog::default()` resuelve `roar` con aridad 1. Verif: `test_catalog_resolves_roar` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC6** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto
**Ya existe** (F0, F1-01 a F1-03): `Action` trait, `ActionResult`, `Catalog`, `Wait`, `Move`, `Say`, `Attack`, `Consume`, `Reproduce` en `crates/instinct-core/src/actions.rs`; eventos `Said`, `Died`, `RuntimeError` en `crates/instinct-core/src/event.rs`; `World` con `ticks`, `events`, `rng` en `crates/instinct-core/src/world.rs`; `Simulation::step_tick` y `advance_tick` en `crates/instinct-core/src/sim.rs`.
**Contratos que debes respetar**: `crates/instinct-core/src/actions.rs` — `apply(world, creature_id, args) -> Result<ActionResult, Diagnostic>`; `roar` consume turno. `crates/instinct-core/src/world.rs` — rejilla por identificadores; eventos en `World.events`.
**Ojo**: `roar` no consume recursos ni mata; solo deja constancia. El "disponible el tick siguiente" se interpreta como: el rugido emitido en el tick N es legible en el tick N+1. En F1 basta con guardar el rugido con su tick; F2 filtra los del tick anterior dentro del radio de visión. La rotación al final del tick (`sim.rs`) evita que un rugido persista indefinidamente.

### Leer primero
- `crates/instinct-core/src/actions.rs` — `Say` para imitar (emite evento, no consume turno en `say`; `roar` sí consume).
- `crates/instinct-core/src/event.rs` — formato de `Said`/`Died` para imitar en `Roared` y su `Display`.
- `crates/instinct-core/src/world.rs` — `emit_event`, `ticks`, `events`.
- `crates/instinct-core/src/sim.rs` — `step_tick` y `advance_tick` para la rotación de rugidos al final del tick.
- `crates/instinct-core/tests/actions_tests.rs` — helpers existentes para imitar.

## Especificación
`roar(value)` recibe un argumento (entero o texto). Emite un evento `Roared { tick, species, pos, text }` donde `text` es el valor mostrado como texto (número → su representación decimal). Almacena el rugido en el mundo con el tick actual para que, en el siguiente tick, F2 pueda leer los rugidos del tick anterior dentro del radio de visión. `roar` consume un turno. `(0,0)` no aplica a `roar` (no hay dirección).

## Pasos sugeridos (opcional)
1. Añade la variante `Roared` a `Event` en `event.rs` con su `Display`.
2. Implementa `Roar` en `actions.rs` emitiendo el evento y guardando el rugido en el mundo.
3. Añade el almacén de rugidos en `World` y el método para añadirlos.
4. En `sim.rs`, rota los rugidos al final de cada tick (los del tick actual se marcan como "del tick anterior").
5. Registra `roar` en `Catalog::default()`.
6. Escribe los tests primero (TDD), luego el código.

## Tests
Nivel: unitario, en `crates/instinct-core/tests/actions_tests.rs`. Casos como comportamientos: emite `Roared` con campos correctos, número como texto, consume turno, almacena con tick, catálogo resuelve `roar`. Qué NO mockear: `World`, `Catalog` reales. Datos/fixtures: `world_with_grass`, `place_walker`.

## Verificación
- `cargo test -p instinct-core --test actions_tests` → pruebas de `roar` pasan.
- `cargo test --workspace` → todo en verde (sin regresiones de F0-F1-03).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si el almacén de rugidos no encaja en `World` sin romper el préstamo inmutable del intérprete, para y reporta en "Notas de implementación" en vez de usar `RefCell` en el mundo entero. Decisiones que NO puedes tomar tú: implementar `roars_near`/`last_roar` (es F2), o hacer que `roar` no consuma turno.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
- AC1 `test_roar_emits_event`: `cargo test -p instinct-core --test actions_tests test_roar_emits_event` → ok. `Roared` con tick, species "Walker", pos (1,1), texto "¡auau!".
- AC2 `test_roar_number_as_text`: `cargo test -p instinct-core --test actions_tests test_roar_number_as_text` → ok. `roar(42)` → texto "42".
- AC3 `test_roar_consumes_turn`: `cargo test -p instinct-core --test actions_tests test_roar_consumes_turn` → ok. `consumes_turn() == true`; vacío consume igualmente.
- AC4 `test_roar_stored_with_tick`: `cargo test -p instinct-core --test actions_tests test_roar_stored_with_tick` → ok. Rugido guardado en `World.roars_current` con `tick == world.ticks`. `test_roar_rotates_to_previous_tick` (sim_tests) → ok: `sim.step_tick()` rota a `roars_previous` y limpia `roars_current`; tick del rugido = 0 = `tick_actual - 1` tras el avance.
- AC5 `test_catalog_resolves_roar`: `cargo test -p instinct-core --test actions_tests test_catalog_resolves_roar` → ok. `roar` aridad 1, consume turno.
- AC6: `cargo test --workspace` → 17 suites, 0 fallos; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all --check` → exit 0.

Cambios: `Event::Roared` + `Display` en `crates/instinct-core/src/event.rs`; `struct Roar`, campos `roars_current`/`roars_previous`, `add_roar`, `previous_roars`, `rotate_roars` en `crates/instinct-core/src/world.rs`; acción `Roar` (aridad 1, consume turno) registrada en `Catalog::default()` en `crates/instinct-core/src/actions.rs`; rotación `self.world.rotate_roars()` al final de cada tick en `step_tick` de `crates/instinct-core/src/sim.rs`. Tests en `crates/instinct-core/tests/actions_tests.rs` y `crates/instinct-core/tests/sim_tests.rs`. Sin stubs/TODOs/código muerto. `roars_near`/`last_roar`/`see`/`name` se dejan para F2; el contrato de PHASE.md (filtro por `tick_actual - 1`) se cumple con el almacén por tick y la rotación.
