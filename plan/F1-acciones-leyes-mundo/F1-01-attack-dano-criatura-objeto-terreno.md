---
id: F1-01
title: attack: daño a criatura, objeto, terreno y casilla propia
phase: F1
status: done
depends_on: []
size: M
---

# F1-01 · attack: daño a criatura, objeto, terreno y casilla propia

## Objetivo
Implementar la acción `attack` que golpea la casilla adyacente en una dirección del mapa: reduce la vida de una criatura, la reserva de un objeto o la reserva del terreno; mata inmediatamente a una criatura que cae a 0 o menos; no hace nada fuera del mapa; registra el daño como evento.

## Alcance
**Haz**
- Añadir el tipo `Attack` que implemente `Action` en `crates/instinct-core/src/actions.rs`.
- Registrar `attack` en `Catalog::default()` con aridad 3.
- `World` guarda los catálogos `terrains` y `objects` (llenados en `World::from_map`) y expone un helper privado/compartido `damage_cell_content(x, y, damage) -> Result<(), Diagnostic>` que lee y muta la reserva del objeto o el terreno de la celda (sin indexar fuera de rango: fuera del mapa = no-op, sin error).
- Daño a criatura en casilla adyacente: pierda `damage` de vida; si llega a 0 o menos, muere en el acto, su casilla queda solo terreno y se emite `Died` causa `vida` en el mismo tick.
- Daño a objeto en casilla adyacente: pierda `damage` de reserva; si llega a 0 o menos, desaparece al final del tick (ley existente).
- Daño a terreno en casilla adyacente (incluida `(0, 0)`, el terreno bajo la propia criatura): pierda `damage` de reserva.
- `damage` puede ser cualquier entero (negativo curaría). La aritmética de salud/reserva se hace con operaciones checked; overflow o underflow se convierten en error de ejecución, nunca en panic.
- Destino fuera del mapa: no hace nada, consume turno, sin error.
- `dx` o `dy` fuera de `{-1, 0, 1}`: error de ejecución nombrando el argumento.
- Argumento de texto: error de ejecución nombrando el argumento.
- Emitir un evento de daño aplicado (`Damaged`) con tick, especie, posición, casilla objetivo y cantidad.
- Tests unitarios que cubran cada caso anterior.
- `cargo test -p instinct-core`, suite completa, clippy y fmt en verde.

**No hagas**
- No implementes `consume`, `reproduce` ni `roar` (son F1-02, F1-03, F1-04).
- No añadas percepciones de cercanía (`roars_near`, `last_roar`); es F2.
- No cambies `Catalog` de forma que rompa `wait`, `move`, `say`.
- No hagas `unwrap`/`expect` sobre datos del usuario ni indexación fuera de rango.

### Archivos previstos
- `crates/instinct-core/src/actions.rs` — modificar — añadir `Attack` y registrarlo; usar helper compartido de golpear casilla.
- `crates/instinct-core/src/event.rs` — modificar — añadir evento `Damaged` (y `Roared` se añade en F1-04).
- `crates/instinct-core/src/world.rs` — modificar — guardar `terrains` y `objects` en `World`; añadir helper seguro para dañar el contenido de una celda (bounds check, sin panic).
- `crates/instinct-core/src/sim.rs` — revisar — la muerte inmediata no requiere cambios: `run_creature_turn` ya ignora criaturas removidas durante el tick.
- `crates/instinct-core/tests/actions_tests.rs` — modificar — añadir tests de `attack`.

## Criterios de aceptación
- [ ] **AC1** `attack` contra criatura adyacente reduce su vida en `damage`; si queda <= 0, muere en el acto, su casilla queda solo terreno y se emite `Died` causa `vida` en el tick actual. Verif: `test_attack_kills_creature_immediately` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC2** `attack` contra criatura con vida suficiente no mata; solo reduce vida y la criatura sigue viva en su casilla. Verif: `test_attack_wounds_creature` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC3** `attack` contra objeto reduce su reserva; al llegar a 0 o menos desaparece al final del tick. Verif: `test_attack_damages_object` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC4** `attack` contra terreno (incluido `(0, 0)`) reduce la reserva del terreno bajo la casilla. Verif: `test_attack_damages_terrain` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC5** `attack` con destino fuera del mapa no hace nada, consume turno y no emite error. Verif: `test_attack_out_of_map_noop` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC6** `attack` con `dx` o `dy` fuera de `{-1, 0, 1}` falla con error que nombra el argumento. Verif: `test_attack_bad_direction_errors` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC7** `attack` con argumento de texto falla con error que nombra el argumento. Verif: `test_attack_text_argument_errors` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC8** `Catalog::default()` resuelve `attack` con aridad 3 y `consumes_turn() == true`. Verif: `test_catalog_resolves_attack` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC9** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto
**Ya existe** (F0): `Action` trait, `ActionResult`, `Catalog`, `Wait`, `Move`, `Say` en `crates/instinct-core/src/actions.rs`; `CellContent`, `Creature`, `World`, `remove_creature`, `move_creature_to`, `end_of_tick_creatures` en `crates/instinct-core/src/world.rs`; `Simulation::step_tick` y `run_creature_turn` en `crates/instinct-core/src/sim.rs`; eventos `Said`, `Died`, `RuntimeError` en `crates/instinct-core/src/event.rs`.
**Contratos que debes respetar**: `crates/instinct-core/src/actions.rs` — acciones son tipos sin estado; `apply` recibe `&mut World`, `CreatureId` y `&[Value]`; devuelve `Result<ActionResult, Diagnostic>`. `crates/instinct-core/src/world.rs` — rejilla guarda identificadores, no referencias; `CellContent::Creature(id)` / `CellContent::Object(idx)` / `CellContent::Empty`; `World` guarda `terrains` y `objects` y expone acceso bounds-checked al contenido de una celda.
**Ojo**: la muerte inmediata ocurre *durante* el turno de quien ataca, no al final del tick; `sim.rs` ya ignora criaturas muertas cuando les llega el turno (`run_creature_turn` retorna temprano si la id ya no existe). Un `attack` sobre la propia casilla `(0, 0)` golpea el terreno bajo la criatura, no a la criatura misma. `sim.rs` no requiere cambios para la muerte inmediata.

### Leer primero
- `crates/instinct-core/src/actions.rs` — patrón de `Move`/`Say` para imitar en `Attack`.
- `crates/instinct-core/src/world.rs` — `cell_is_free`, `remove_creature`, `creature_position` y `CellContent`.
- `crates/instinct-core/src/sim.rs` — `run_creature_turn` y `die_by_error` para ver cómo se maneja una muerte durante el turno.
- `crates/instinct-core/src/event.rs` — formato de `Died` para el nuevo evento de daño.
- `crates/instinct-core/tests/actions_tests.rs` — helpers `world_with_grass`, `rock_world`, `place_walker` para reutilizar en tests.

## Especificación
La acción `attack` recibe tres enteros. `dx` y `dy` en `{-1,0,1}`; cualquier otro valor es error nombrando el argumento. `damage` puede ser cualquier entero (negativo curaría, pero no se limita). La casilla objetivo es `(x+dx, y+dy)` de la criatura atacante. Si está fuera del mapa: no-op. Si hay criatura: le resta `damage` de vida (aritmética checked; overflow/underflow = error de ejecución); si vida <= 0, se retira del mundo inmediatamente (casilla libre) y `Died` causa `vida`. Si hay objeto: le resta `damage` de reserva (checked); si <= 0, desaparece al final del tick (ley existente). Si hay solo terreno: el terreno bajo esa casilla resta `damage` de reserva (checked). `(0, 0)` apunta al terreno bajo la propia criatura. El daño a reservas se aplica a través del helper de `World` que lee/muta los catálogos `terrains`/`objects` con bounds check; nunca panica por índice fuera de rango.

## Pasos sugeridos (opcional)
1. Añade el evento `Damaged` a `event.rs` y su `Display`.
2. Añade a `World` los campos `terrains` y `objects` (llenados en `from_map`) y un helper seguro para dañar el contenido de una celda (bounds check, sin panic).
3. Implementa `Attack` en `actions.rs` siguiendo el patrón de `Move`, usando el helper compartido de "golpear casilla adyacente" y aritmética checked.
4. Registra `attack` en `Catalog::default()`.
5. `sim.rs` no requiere cambios: la muerte inmediata ya está manejada (`run_creature_turn` ignora criaturas removidas durante el tick).
6. Escribe los tests primero (TDD), luego el código, y ejecuta `cargo test -p instinct-core --test actions_tests`.

## Tests
Nivel: unitario, en `crates/instinct-core/tests/actions_tests.rs`. Casos como comportamientos: criatura muere al golpe (mismo tick), criatura herida no muere, objeto pierde reserva, terreno pierde reserva, fuera del mapa no-op, dirección inválida error, texto error, catálogo resuelve `attack`. Qué NO mockear: el `World` real y el `Catalog` real. Determinismo: semilla fija no aplica aquí (no hay `random` en `attack`); usar mundos construidos a mano. Datos/fixtures: reutilizar `world_with_grass` y `rock_world`; crear variantes con criaturas y objetos.

## Verificación
- `cargo test -p instinct-core --test actions_tests` → pruebas de `attack` pasan.
- `cargo test --workspace` → todo en verde (sin regresiones de F0).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si el repo contradice este issue (por ejemplo, `World` no expone lo necesario para dañar una casilla), para y reporta en "Notas de implementación" en vez de improvisar. Decisiones que NO puedes tomar tú: cambiar la firma del trait `Action`, mover la muerte inmediata al final del tick, o añadir percepciones de cercanía.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
- AC1 `test_attack_kills_creature_immediately`: `cargo test -p instinct-core --test actions_tests test_attack_kills_creature_immediately` → ok. Criatura muere al golpe, casilla queda `Empty`, `Died` causa `vida` mismo tick, atacante vivo.
- AC2 `test_attack_wounds_creature`: `cargo test -p instinct-core --test actions_tests test_attack_wounds_creature` → ok. Vida reducida, criatura sigue en su casilla, sin `Died`.
- AC3 `test_attack_damages_object`: `cargo test -p instinct-core --test actions_tests test_attack_damages_object` → ok. Reserva del objeto reducida; al 0 sigue en celda (desaparece al final del tick por ley existente).
- AC4 `test_attack_damages_terrain`: `cargo test -p instinct-core --test actions_tests test_attack_damages_terrain` → ok. Terreno adyacente y casilla propia (dx=0,dy=0) reducen reserva del terreno de la celda.
- AC5 `test_attack_out_of_map_noop`: `cargo test -p instinct-core --test actions_tests test_attack_out_of_map_noop` → ok. Fuera del mapa: no-op, sin error, sin evento.
- AC6 `test_attack_bad_direction_errors`: `cargo test -p instinct-core --test actions_tests test_attack_bad_direction_errors` → ok. dx/dy fuera de {-1,0,1} error nombrando el argumento.
- AC7 `test_attack_text_argument_errors`: `cargo test -p instinct-core --test actions_tests test_attack_text_argument_errors` → ok. Argumento de texto error nombrando el argumento (dx, dy, damage).
- AC8 `test_catalog_resolves_attack`: `cargo test -p instinct-core --test actions_tests test_catalog_resolves_attack` → ok. `attack` aridad 3, `consumes_turn()==true`.
- AC9 `cargo test --workspace` → todo verde (0 fallos); `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all --check` → exit 0.
Cambios: `Attack` registrado en `Catalog::default()` (`crates/instinct-core/src/actions.rs`); evento `Damaged` en `crates/instinct-core/src/event.rs`; `World` guarda `terrains`/`objects` (clonados en `from_map`) y helper `damage_cell_content` con bounds-check y aritmética checked (`crates/instinct-core/src/world.rs`); tests en `crates/instinct-core/tests/actions_tests.rs`. La casilla propia (0,0) golpea el terreno bajo la criatura, no a la criatura. `sim.rs` sin cambios (muerte inmediata ya gestionada).
