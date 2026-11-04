---
id: F1-02
title: consume: igual que attack más curación del atacante
phase: F1
status: todo
depends_on: [F1-01]
size: M
---

# F1-02 · consume: igual que attack más curación del atacante

## Objetivo
Implementar la acción `consume` que actúa como `attack` sobre la casilla adyacente y, además, la criatura atacante gana `heal` puntos de vida (sin tope).

## Alcance
**Haz**
- Añadir el tipo `Consume` que implemente `Action` en `crates/instinct-core/src/actions.rs`.
- Registrar `consume` en `Catalog::default()` con aridad 4.
- Reusar la lógica compartida de "golpear casilla adyacente" de `attack` (F1-01) a través del helper de `World`: daño a criatura/objeto/terreno, muerte inmediata de criatura, fuera del mapa no-op, `(0, 0)` = terreno bajo la propia criatura. La aritmética de salud/reserva/curación se hace con operaciones checked; overflow/underflow = error de ejecución, nunca panic.
- Después de aplicar el daño, la criatura atacante gana `heal` de vida (sin tope; `health` puede acumularse). `heal` se aplica a la atacante incluso si el daño no afectó a nada salvo el caso fuera del mapa (ver nota abajo).
- `dx`/`dy` fuera de `{-1, 0, 1}`: error nombrando el argumento.
- Argumento de texto (cualquiera de los 4): error nombrando el argumento.
- Emitir evento de daño aplicado y, opcionalmente, evento de curación; `consume` consume turno aunque falle.
- Tests unitarios que cubran daño + heal, objetivo fuera del mapa no-op, y errores de argumento.
- `cargo test -p instinct-core`, suite completa, clippy y fmt en verde.

**No hagas**
- No dupliques la lógica de golpear casilla: `consume` debe llamar/compartir con `attack` el camino común (helper de `World`).
- No implementes `reproduce` ni `roar` (F1-03, F1-04).
- No añadas límite máximo de vida (el enunciado dice que no hay vida máxima).
- No hagas `unwrap`/`expect` sobre datos del usuario ni indexación fuera de rango.

### Archivos previstos
- `crates/instinct-core/src/actions.rs` — modificar — añadir `Consume` y registrarlo; refactor opcional para compartir lógica con `Attack`.
- `crates/instinct-core/src/event.rs` — modificar — si se añade evento de curación (opcional).
- `crates/instinct-core/tests/actions_tests.rs` — modificar — añadir tests de `consume`.

## Criterios de aceptación
- [ ] **AC1** `consume` contra criatura reduce su vida en `damage`; si queda <= 0, muere en el acto y su casilla queda solo terreno. Verif: `test_consume_damages_creature` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC2** `consume` cura a la atacante en `heal` (su vida sube `heal`, sin tope). Verif: `test_consume_heals_attacker` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC3** `consume` contra objeto reduce su reserva; al llegar a 0 o menos desaparece al final del tick. Verif: `test_consume_damages_object` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC4** `consume` contra terreno reduce la reserva del terreno bajo la casilla (incluido `(0, 0)`). Verif: `test_consume_damages_terrain` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC5** `consume` con destino fuera del mapa no hace nada, consume turno, sin error. Verif: `test_consume_out_of_map_noop` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC6** `consume` con `dx`/`dy` fuera de `{-1, 0, 1}` falla con error que nombra el argumento. Verif: `test_consume_bad_direction_errors` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC7** `consume` con argumento de texto falla con error que nombra el argumento. Verif: `test_consume_text_argument_errors` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC8** `Catalog::default()` resuelve `consume` con aridad 4 y `consumes_turn() == true`. Verif: `test_catalog_resolves_consume` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC9** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto
**Ya existe** (F0 y F1-01): `Action` trait, `ActionResult`, `Catalog`, `Wait`, `Move`, `Say`, `Attack` (F1-01) en `crates/instinct-core/src/actions.rs`; `CellContent`, `Creature`, `World`, `remove_creature`, `end_of_tick_creatures` en `crates/instinct-core/src/world.rs`; eventos en `crates/instinct-core/src/event.rs`; tests de `attack` en `crates/instinct-core/tests/actions_tests.rs`.
**Contratos que debes respetar**: `crates/instinct-core/src/actions.rs` — el trait `Action` con `apply` que recibe el mundo, el id de la criatura y los argumentos ya evaluados; `consume` consume turno. `crates/instinct-core/src/world.rs` — rejilla por identificadores; `CellContent::Creature(id)` / `CellContent::Object(idx)` / `CellContent::Empty`.
**Ojo**: `heal` se aplica a la *atacante* incluso si el daño no afectó a nada (fuera del mapa aún consume turno; la pregunta de si se cura sin objetivo queda a criterio — el enunciado dice "Igual que attack, y además la criatura gana heal", así que si attack no hace nada fuera del mapa, consume tampoco hace nada, incluida la curación). `damage` y `heal` pueden ser negativos sin restricción del lenguaje.

### Leer primero
- `crates/instinct-core/src/actions.rs` — `Attack` recién añadido (F1-01) y el camino compartido de golpear casilla.
- `crates/instinct-core/src/world.rs` — `creature_position`, `remove_creature`, `CellContent`.
- `crates/instinct-core/tests/actions_tests.rs` — helpers existentes y tests de `attack` para imitar.

## Especificación
La acción `consume` recibe cuatro enteros. `dx` y `dy` en `{-1,0,1}`; cualquier otro valor es error nombrando el argumento. Primero actúa exactamente como `attack` sobre la casilla adyacente `(x+dx, y+dy)`. Luego, la criatura atacante suma `heal` a su vida actual (sin tope). Si la casilla objetivo está fuera del mapa, `consume` no hace nada (ni daño ni curación) y consume turno. `(0, 0)` apunta al terreno bajo la propia criatura: el terreno pierde `damage` de reserva y la atacante gana `heal` de vida.

## Pasos sugeridos (opcional)
1. Extrae a una función privada compartida la lógica "aplicar daño a la entidad de la casilla adyacente" usada por `Attack`.
2. Implementa `Consume` llamando a esa función compartida y luego aplicando `heal` a la atacante.
3. Registra `consume` en `Catalog::default()`.
4. Escribe los tests primero (TDD), luego el código, ejecuta `cargo test -p instinct-core --test actions_tests`.

## Tests
Nivel: unitario, en `crates/instinct-core/tests/actions_tests.rs`. Casos como comportamientos: daño + curación a atacante, criatura objetivo muere, objeto pierde reserva, terreno pierde reserva, fuera del mapa no-op (ni daño ni curación), dirección inválida error, texto error, catálogo resuelve `consume`. Qué NO mockear: `World` y `Catalog` reales. Datos/fixtures: reutilizar `world_with_grass`, `rock_world`, `place_walker`.

## Verificación
- `cargo test -p instinct-core --test actions_tests` → pruebas de `consume` pasan.
- `cargo test --workspace` → todo en verde (sin regresiones de F0 y F1-01).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si el camino compartido de golpear casilla no es fácilmente reutilizable, para y reporta en "Notas de implementación" en vez de duplicar lógica. Decisiones que NO puedes tomar tú: añadir tope de vida máximo, curar aunque el objetivo esté fuera del mapa, o cambiar la firma del trait `Action`.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
<!-- Lo rellena el implementador: desviaciones, hechos descubiertos que afectan a issues siguientes, deuda consciente. -->
