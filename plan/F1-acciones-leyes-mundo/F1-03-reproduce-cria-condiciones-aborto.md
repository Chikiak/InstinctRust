---
id: F1-03
title: reproduce: cría, condiciones y aborto natural
phase: F1
status: done
depends_on: [F1-01]
size: M
---

# F1-03 · reproduce: cría, condiciones y aborto natural

## Objetivo
Implementar la acción `reproduce` que crea una cría de la misma especie en la casilla adyacente si está libre y la madre tiene más de `health` de vida; la madre pierde `health`, la cría nace con edad 0, sin variables y arranca en `start:`; su primer turno es el tick siguiente. Cualquier fallo es aborto natural (no hace nada).

## Alcance
**Haz**
- Añadir el tipo `Reproduce` que implemente `Action` en `crates/instinct-core/src/actions.rs`.
- Registrar `reproduce` en `Catalog::default()` con aridad 3.
- Casilla objetivo `(x+dx, y+dy)` de la madre: debe estar dentro del mapa y ser solo terreno (`CellContent::Empty`); si hay objeto, criatura o está fuera del mapa, aborto natural (no hace nada, consume turno).
- La madre debe tener vida estrictamente mayor que `health` (`mother.health > health`); si no, aborto natural.
- La madre pierde `health` de vida (puede quedar en 0 o menos; la muerte por end-of-tick la procesa al final del tick, no inmediatamente).
- La cría nace en la casilla objetivo con: misma especie (mismo nombre, facción, programa), vida = `health`, edad 0, sin turnos de sueño, puntero de ejecución al inicio (`start:`), variables vacías (no hereda nada de la madre). Como `place_creature` inicializa la salud con `species.health`, `reproduce` debe ajustar la salud de la cría a `health` tras colocarla (añadir un método público en `World` para fijar la salud de una criatura, o exponer acceso controlado; nunca indexar fuera de rango ni hacer `unwrap`/`expect`).
- La cría recibe un `CreatureId` nuevo y estable; su primer turno es el tick siguiente al de nacimiento (el snapshot de `step_tick` la excluye del tick actual).
- Semántica de cría con salud <= 0: la condición `mother.health > health` puede cumplirse y la cría nace con vida <= 0; al final del tick `end_of_tick_creatures` le aplica -1 de vida y puede morir por `vida` ese mismo tick. Este comportamiento es intencional y debe reflejarse en los tests.
- Emitir evento de nacimiento (reutilizar `Died` no aplica; considerar un evento nuevo o `Said`-like, o bien no emitir evento y verificar por estado del mundo). Decisión abierta en PHASE.md: se evalúa si añadir evento `Born` o verificar solo por estado.
- `dx`/`dy` fuera de `{-1, 0, 1}`: error nombrando el argumento.
- Argumento de texto: error nombrando el argumento.
- `reproduce` consume turno aunque sea aborto natural.
- Tests unitarios que cubran cría exitosa, aborto por casilla ocupada, aborto por vida insuficiente, aborto fuera del mapa, cría sin variables y arranque en `start:`.
- `cargo test -p instinct-core`, suite completa, clippy y fmt en verde.

**No hagas**
- No implementes herencia de variables de la madre (la cría nace sin variables).
- No hagas que la cría juegue en el mismo tick (primer turno al siguiente).
- No implementes `roar` (F1-04).
- No hagas `unwrap`/`expect` sobre datos del usuario ni indexación fuera de rango.

### Archivos previstos
- `crates/instinct-core/src/actions.rs` — modificar — añadir `Reproduce` y registrarlo.
- `crates/instinct-core/src/event.rs` — modificar — si se añade evento de nacimiento (opcional, decisión de PHASE.md).
- `crates/instinct-core/src/world.rs` — modificar — `place_creature` para la cría; añadir método público para fijar la salud de una criatura (la cría nace con `species.health` y debe ajustarse a `health`).
- `crates/instinct-core/src/sim.rs` — revisar — la cría añadida durante un turno no debe jugar ese mismo tick (snapshot de ids al inicio de `step_tick`).
- `crates/instinct-core/tests/actions_tests.rs` — modificar — añadir tests de `reproduce`.

## Criterios de aceptación
- [ ] **AC1** `reproduce` sobre casilla solo terreno y madre con vida > `health` crea cría en esa casilla con vida = `health`, edad 0, sin variables y puntero al inicio. Verif: `test_reproduce_creates_offspring` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC2** La madre pierde `health` de vida tras reproducirse. Verif: `test_reproduce_mother_loses_health` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC3** La cría no hereda variables de la madre (mapa de variables vacío). Verif: `test_reproduce_offspring_no_variables` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC4** La cría arranca en `start:` y su primer turno es el tick siguiente. Verif: `test_reproduce_offspring_starts_at_start_next_tick` | `cargo test -p instinct-core --test sim_tests` o `actions_tests` | prueba pasa (simulación de un tick).
- [ ] **AC5** `reproduce` sobre casilla con objeto o criatura: aborto natural, no hay cría. Verif: `test_reproduce_abort_occupied_cell` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC6** `reproduce` fuera del mapa: aborto natural, no hay cría. Verif: `test_reproduce_abort_out_of_map` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC7** `reproduce` con madre vida <= `health`: aborto natural, no hay cría. Verif: `test_reproduce_abort_insufficient_health` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC8** `Catalog::default()` resuelve `reproduce` con aridad 3 y `consumes_turn() == true`. Verif: `test_catalog_resolves_reproduce` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC9** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto
**Ya existe** (F0, F1-01, F1-02): `Action` trait, `ActionResult`, `Catalog`, `Wait`, `Move`, `Say`, `Attack`, `Consume` en `crates/instinct-core/src/actions.rs`; `World::place_creature`, `cell_is_free`, `creature_position`, `CellContent` en `crates/instinct-core/src/world.rs`; `Simulation::step_tick` hace snapshot de ids vivos al inicio del tick (una cría nacida durante el turno no juega ese tick) en `crates/instinct-core/src/sim.rs`; `Species` (nombre, facción, programa) en `crates/instinct-core/src/species.rs`.
**Contratos que debes respetar**: `crates/instinct-core/src/actions.rs` — el trait `Action` con `apply` que recibe el mundo, el id de la criatura y los argumentos ya evaluados. `crates/instinct-core/src/world.rs` — `place_creature` ya valida bounds y celda libre. `crates/instinct-core/src/species.rs` — `Species` clona para la cría (misma especie, facción, programa).
**Ojo**: `place_creature` ya rechaza celda ocupada y fuera del mapa, así que `reproduce` puede usarlo directamente para el aborto natural. La madre puede quedar con vida <= 0 tras parir; no muere inmediatamente (la muerte es al final del tick por `end_of_tick_creatures`). La cría nace con `health` indicado en la acción, no con la vida de la madre.

### Leer primero
- `crates/instinct-core/src/actions.rs` — `Attack`/`Consume` y el camino compartido de golpear casilla.
- `crates/instinct-core/src/world.rs` — `place_creature` y `CellContent::Empty`.
- `crates/instinct-core/src/species.rs` — `Species::new` y campos (nombre, facción, health, vision, lifespan, program).
- `crates/instinct-core/src/sim.rs` — `step_tick` snapshot de ids (por qué la cría no juega el tick de nacimiento).
- `crates/instinct-core/tests/actions_tests.rs` — helpers para reutilizar.

## Especificación
La acción `reproduce` recibe tres enteros. `dx` y `dy` en `{-1,0,1}`. Casilla objetivo = `(x+dx, y+dy)` de la madre. Condiciones para éxito: objetivo dentro del mapa Y `CellContent::Empty` Y `mother.health > health`. Si falla cualquiera: aborto natural (no hace nada, consume turno). Si cumple: madre `health -= health_param`; cría nace en objetivo con copia de `Species` de la madre, vida = `health_param`, edad 0, sin turnos de sueño, puntero al inicio, variables vacías, nuevo `CreatureId`. La cría no aparece en el snapshot de turnos de este tick (su primer turno es el siguiente). `health` puede ser cualquier entero; si es <= 0, la condición `mother.health > health` puede cumplirse fácilmente y la cría nace con vida <= 0 (morirá al final del tick).

## Pasos sugeridos (opcional)
1. Implementa `Reproduce` en `actions.rs` usando `World::place_creature` para la cría (ello ya valida celda libre y bounds).
2. Registra `reproduce` en `Catalog::default()`.
3. Escribe los tests primero (TDD), incluido uno de simulación de un tick para verificar que la cría no juega al nacer.
4. Ejecuta `cargo test -p instinct-core --test actions_tests`.

## Tests
Nivel: unitario, en `crates/instinct-core/tests/actions_tests.rs`; para AC4 (primer turno siguiente tick) puede usarse `crates/instinct-core/tests/sim_tests.rs`. Casos como comportamientos: cría exitosa (vida, edad 0, sin variables, pc 0), madre pierde vida, aborto celda ocupada, aborto fuera del mapa, aborto vida insuficiente, cría no juega el tick de nacimiento. Qué NO mockear: `World`, `Catalog`, `Species` reales. Datos/fixtures: `world_with_grass`, `rock_world`, `place_walker`; crear una especie con programa compilado para la cría.

## Verificación
- `cargo test -p instinct-core --test actions_tests` → pruebas de `reproduce` pasan.
- `cargo test --workspace` → todo en verde (sin regresiones).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si `place_creature` no encaja porque la cría necesita el programa compilado de la especie, para y reporta en "Notas de implementación" en vez de saltarte la validación. Decisiones que NO puedes tomar tú: herencia de variables, cría jugando el mismo tick, o cambiar la firma del trait `Action`.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
- AC1 `test_reproduce_creates_offspring`: `cargo test -p instinct-core --test actions_tests test_reproduce_creates_offspring` → ok. Cría en (1,2) con vida 5 (= health), edad 0, sleep_turns 0, pc 0, variables vacías, misma especie/facción/programa; madre en su casilla.
- AC2 `test_reproduce_mother_loses_health`: `cargo test -p instinct-core --test actions_tests test_reproduce_mother_loses_health` → ok. Madre 10 → 5.
- AC3 `test_reproduce_offspring_no_variables`: `cargo test -p instinct-core --test actions_tests test_reproduce_offspring_no_variables` → ok. Cría sin variables; madre conserva las suyas.
- AC4 `test_reproduce_offspring_starts_at_start_next_tick`: `cargo test -p instinct-core --test sim_tests test_reproduce_offspring_starts_at_start_next_tick` → ok. Tras el tick de nacimiento: cría en (1,2), vida 4 (5 - 1 fin de tick), edad 1, pc 0 (no jugó el tick de nacimiento por el snapshot de `step_tick`). Tick siguiente: pc 2, ejecutó su turno.
- AC5 `test_reproduce_abort_occupied_cell` + `test_reproduce_abort_occupied_by_object`: `cargo test -p instinct-core --test actions_tests` → ok. Celda ocupada por criatura u objeto: aborto natural, sin cría, madre sin perder vida.
- AC6 `test_reproduce_abort_out_of_map`: `cargo test -p instinct-core --test actions_tests test_reproduce_abort_out_of_map` → ok. Fuera del mapa: aborto natural.
- AC7 `test_reproduce_abort_insufficient_health`: `cargo test -p instinct-core --test actions_tests test_reproduce_abort_insufficient_health` → ok. `mother.health <= health` (incluido `==`): aborto natural.
- AC8 `test_catalog_resolves_reproduce`: `cargo test -p instinct-core --test actions_tests test_catalog_resolves_reproduce` → ok. `reproduce` aridad 3, `consumes_turn() == true`.
- AC9 `cargo test --workspace` → todo verde (0 fallos en todos los crates); `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings; `cargo fmt --all --check` → exit 0; `cargo build --workspace` → exit 0.
- Decisiones: no se añade evento `Born` (PHASE.md no lo lista); se verifica por estado del mundo, como permite el issue. La cría nace con `species.health` vía `place_creature` y se ajusta a `health` con el nuevo método público `World::set_creature_health`. La cría está sujeta a las leyes de fin de tick del tick de nacimiento (puede morir por `vida` ese mismo tick si `health <= 0`, comportamiento intencional del enunciado). `dx`/`dy` fuera de `{-1,0,1}` y argumento de texto fallan nombrando el argumento; cualquier fallo de condiciones consume turno (no hace nada, devuelve `Ok`).
- Archivos: `crates/instinct-core/src/actions.rs` (`Reproduce`, registro en `Catalog::default()`); `crates/instinct-core/src/world.rs` (`World::set_creature_health`); `crates/instinct-core/tests/actions_tests.rs` (8 tests de `reproduce`); `crates/instinct-core/tests/sim_tests.rs` (test AC4). Sin `unwrap`/`expect` sobre datos de usuario, sin indexación fuera de rango, sin floats, sin dependencias nuevas.
- Nota: `plan/.../NOCHE_F1.md` tiene una modificación de despacho pre-existente (no de este issue); se deja fuera del commit de este issue.
