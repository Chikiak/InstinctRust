---
id: F2-06
title: Mundo y tick: aritmética comprobada y coordenadas sin envoltura
phase: F2
status: done
depends_on: []
size: M
---

# F2-06 · Mundo y tick: aritmética comprobada y coordenadas sin envoltura

## Objetivo

Cerrar los problemas W3 y W4 y la cobertura W6: que ningún cálculo del mundo pueda desbordar en
silencio ni convertir una coordenada negativa en un índice enorme, y que las entradas inválidas del
mundo y de la simulación tengan prueba con resultado observable.

## Alcance

**Haz**

- En `crates/instinct-core/src/actions.rs`, `Move::apply`: comprobar el signo antes de convertir
  coordenadas a `usize` (hoy `nx as usize` / `ny as usize`, líneas 402-415). La comprobación debe
  ser un helper pequeño y determinista que convierta entero a índice rechazando negativos, con su
  propia prueba unitaria.
- En `crates/instinct-core/src/world.rs`, `end_of_tick_creatures` (líneas 496-530): usar
  `checked_sub`/`checked_add` para `health - 1` y `age + 1`. El desbordamiento con `i64::MIN` o
  `i64::MAX` se convierte en error de ejecución o evento, nunca en pánico en debug.
- Ampliar `crates/instinct-core/tests/world_tests.rs` y `crates/instinct-core/tests/sim_tests.rs`
  con la cobertura W6: `damage_cell_content` con desbordamiento y no-op fuera del mapa;
  `move_creature_to` con identificador inexistente y destino fuera de rango; `apply_action_result`
  con identificador desconocido; `set_creature_health` y `set_variable` con identificador
  desconocido; fin de tick con vida en `i64::MIN`.
- `cargo test -p instinct-core`, clippy y fmt en verde.

**No hagas**

- No cambies quién es el dueño de `terrains`/`objects` (eso es F2-07, problema W2).
- No sustituyas el `Vec<Creature>` por almacenes indexados (eso es F4-04, problema W5). Aquí basta
  con que `dead_ids.contains` deje de consultarse dentro del bucle si y solo si sale gratis al
  reescribir el bucle; si no, déjalo y anótalo.
- No toques la CLI ni el intérprete.

### Archivos previstos

- `crates/instinct-core/src/actions.rs` — modificar — `Move::apply` y el helper de conversión.
- `crates/instinct-core/src/world.rs` — modificar — aritmética comprobada en fin de tick.
- `crates/instinct-core/tests/world_tests.rs`, `crates/instinct-core/tests/sim_tests.rs` — modificar — cobertura W6.

## Criterios de aceptación

- [ ] **AC1** Ninguna coordenada negativa se convierte en índice: existe un helper de conversión con prueba unitaria que rechaza negativos y `Move::apply` lo usa. Verif: `test_index_from_i64_rejects_negative` | `cargo test -p instinct-core --test actions_tests` | prueba pasa.
- [ ] **AC2** `end_of_tick_creatures` con vida en `i64::MIN` (o edad en el máximo) no pánico ni en debug. Verif: `test_end_of_tick_min_health_no_panic` | `cargo test -p instinct-core --test world_tests` | prueba pasa.
- [ ] **AC3** `damage_cell_content` cubre desbordamiento y no-op fuera del mapa. Verif: `test_damage_cell_overflow_and_out_of_map` | `cargo test -p instinct-core --test world_tests` | prueba pasa.
- [ ] **AC4** `move_creature_to` con identificador inexistente y destino fuera de rango devuelve error controlado. Verif: `test_move_creature_invalid_inputs_error` | `cargo test -p instinct-core --test world_tests` | prueba pasa.
- [ ] **AC5** `apply_action_result`, `set_creature_health` y `set_variable` con identificador desconocido no pánico. Verif: `test_world_setters_unknown_id_error` | `cargo test -p instinct-core --test world_tests` | prueba pasa.
- [ ] **AC6** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `World::damage_cell_content`, `move_creature_to`, `apply_action_result`,
`set_creature_health`, `set_variable`, `end_of_tick_creatures` y `end_of_tick_terrain_and_objects`
en `crates/instinct-core/src/world.rs`; `Move::apply` con validación de dirección en
`{-1, 0, 1}` en `crates/instinct-core/src/actions.rs`; `Catalog` y las acciones de F1; los helpers
`world_with_grass`, `rock_world` y `place_walker` en `actions_tests.rs` reutilizables.
**Contratos que debes respetar**: la rejilla guarda identificadores, no referencias; fuera del mapa
es no-op sin error en las acciones; aritmética comprobada en la ruta de daño y curación ya
existente; el mundo no imprime.
**Ojo**: `Move::apply` valida la dirección en `{-1, 0, 1}`, así que el camino del cast negativo es
defensivo: el AC se cumple con un helper comprobable y usado, no con un caso alcanzable desde el
lenguaje.

### Leer primero

- `crates/instinct-core/src/world.rs` — `end_of_tick_creatures`, `damage_cell_content`, `move_creature_to`, los setters.
- `crates/instinct-core/src/actions.rs` — `Move::apply` y `validate_direction`.
- `crates/instinct-core/tests/world_tests.rs`, `crates/instinct-core/tests/sim_tests.rs` — estilo y helpers.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` sección 4 (W1 a W6).

## Especificación

Un turno de fin de tick calcula la vida y la edad siguientes con aritmética comprobada; si no se
puede calcular, el mundo no pánico: se emite evento o error de ejecución según el contrato que ya
tiene la muerte. Toda conversión de coordenada entera a índice pasa por un helper que rechaza
negativos. Las entradas inválidas del mundo devuelven `Err(Diagnostic)` o no-op documentado, nunca
pánico.

## Pasos sugeridos (opcional)

1. Escribe las pruebas W6 primero; ejecútalas para ver el fallo actual.
2. Añade el helper de conversión con su prueba y úsalo en `Move::apply`.
3. Cambia la aritmética de fin de tick a comprobada.
4. Completa la cobertura W6 y ejecuta `cargo test -p instinct-core`.

## Tests

Nivel: unitario, en `world_tests.rs`, `sim_tests.rs` y `actions_tests.rs`. Casos como
comportamientos: vida en el mínimo no pánico, daño con desbordamiento, entradas fuera de rango o con
identificador desconocido. Qué NO mockear: el `World` real. Determinismo: semilla fija cuando la
prueba pase por la simulación. Datos: mundos construidos a mano con los helpers existentes.

## Verificación

- `cargo test -p instinct-core` → todas las pruebas del núcleo pasan.
- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si el desbordamiento de fin de tick exige decidir entre evento y error de ejecución, elige error de
ejecución (mata a esa criatura) y documéntalo; no cambies la ley de -1 de vida y +1 de edad.
Decisiones que NO puedes tomar tú: cambiar el dueño de los catálogos (F2-07) ni reemplazar el
almacén de criaturas (F4-04).

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

- Decisión de desbordamiento de fin de tick: `health - 1` y `age + 1` usan
  `checked_sub`/`checked_add` en `end_of_tick_creatures`. Cuando desbordan
  (`i64::MIN - 1` o `i64::MAX + 1`) la criatura muere con un evento `Died`
  causa `"error"`; nunca hay pánico. La ley de -1 de vida y +1 de edad se
  conserva para sobrevivientes.
- AC1 (PASS): `pub fn index_from_i64(n: i64) -> Option<usize>` en
  `crates/instinct-core/src/actions.rs`, usado por `Move::apply` antes de
  convertir coordenadas a `usize`; prueba `test_index_from_i64_rejects_negative`
  en `tests/actions_tests.rs` (`cargo test -p instinct-core --test actions_tests` → ok).
- AC2 (PASS): `test_end_of_tick_min_health_no_panic` en `tests/world_tests.rs`
  (`cargo test -p instinct-core --test world_tests` → ok).
- AC3 (PASS): `test_damage_cell_overflow_and_out_of_map` en
  `tests/world_tests.rs`; `damage_cell_content` visibilizado a `pub` para
  poder probarlo desde la integración (`cargo test -p instinct-core --test world_tests` → ok).
- AC4 (PASS): `test_move_creature_invalid_inputs_error` en `tests/world_tests.rs`;
  `move_creature_to` rechaza destino fuera del mapa con `Err` en lugar de pánico
  (`cargo test -p instinct-core --test world_tests` → ok).
- AC5 (PASS): `test_world_setters_unknown_id_error` en `tests/world_tests.rs`
  (`apply_action_result`, `set_creature_health`, `set_variable` con id desconocido
  devuelven `Err`, sin pánico) (`cargo test -p instinct-core --test world_tests` → ok).
- AC6 (PASS): `cargo test --workspace` → todo verde; `cargo clippy --workspace
  --all-targets -- -D warnings` → exit 0 sin warnings; `cargo fmt --all --check`
  → exit 0.
- Rama: `feat/f2-06-mundo-aritmetica`. Merge `develop` fast-forward (sin
  conflictos; ningún archivo de mi alcance fue tocado por F2-01/F2-02).
- Pruebas existentes ajustadas: ninguna; solo se añadieron las 5 pruebas W6.
- Firmas cambiadas: `damage_cell_content` de `pub(crate)` a `pub` (necesario
  para la prueba AC3 desde `tests/`). Ninguna otra firma pública modificada.
