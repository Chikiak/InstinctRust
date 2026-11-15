---
id: R1-07
title: Partir tests largos en módulos ya-split
phase: R1
status: done
depends_on: [R1-06]
size: L
---

# R1-07 · Partir tests largos en módulos ya-split

## Objetivo

Ocho módulos de prueba ya viven en carpetas, pero contienen tests de 60-140 líneas que apilan
escenarios en una sola función. Partirlos en tests enfocados y helpers compartidos, sin cambiar
una sola aserción.

## Alcance

**Haz** (mover/splitear; los tests resultantes deben afirmar exactamente lo mismo, repartido)
- `tests/actions/reproduce.rs` (~290): el test de ~67 líneas (L20-87) → varios tests o helpers.
- `tests/actions/consume.rs` (~220): el test de ~61 líneas (L173-234) → dividir.
- `tests/actions/move_.rs` (~190): el test de ~112 líneas (L9-121) → dividir.
- `tests/sim/determinism.rs` (~215): el test de ~82 líneas (L142-224) → dividir.
- `tests/sim/roars.rs` (~195): tests de ~66 y ~82 líneas (L54-120, L121-203) → dividir.
- `tests/perceptions/nearby.rs` (~190): test de ~83 líneas (L60-143) → dividir.
- `tests/perceptions/catalog.rs` (~185): test de ~106 líneas (L8-114) → dividir.
- `tests/program/line_clone.rs` (~165): test único de ~142 líneas → varios tests/helpers.
- Helpers comunes vía `common.rs` de cada carpeta (ya existen; no los cruces entre carpetas).

**No hagas**
- No cambies datos de entrada ni expectativas; si al dividir parece que sobra una aserción,
  se mantiene.
- No cambies nombres de tests salvo que el split lo exija; si renombras, que el nombre siga
  describiendo el mismo comportamiento.
- No toques `attack.rs`, `compile/names.rs`, `interp/errors.rs` (ya tienen tests cortos).

### Archivos previstos
- Los ocho archivos anteriores — modificar (dividir tests, añadir helpers).
- `common.rs` de las carpetas afectadas — modificar si hace falta un helper compartido.

## Criterios de aceptación

- [ ] **AC1** Ningún archivo de prueba de la lista queda con un test de más de ~60 líneas. Verif:
  lectura + `git show --stat HEAD`.
- [ ] **AC2** Mismo número de tests en verde que antes o más (nunca menos aserciones). Verif:
  `cargo test --workspace 2>&1 | grep "test result"` y comparación de totales.
- [ ] **AC3** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.

## Contexto

Auditoría: los archivos ya están split; la deuda son tests gigantes, no estructura de archivos.

### Leer primero

Los ocho archivos, sus `mod.rs` y `common.rs`.

## Notas de implementación

Rama: `feat/r1-07-partir-tests-largos` (partiendo de `develop`).

AC1 (ningún test > ~60 líneas): verificado por lectura; todos los tests resultantes están por debajo de ~60 líneas.
AC2 (mismo número o más tests verdes, nunca menos aserciones): los 9 tests largos se reemplazaron por 30 tests enfocados (net +21) en los 8 módulos; todas las aserciones y datos de entrada se conservan exactamente, repartidos.
AC3 (clippy/fmt): `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0.

Tests resultantes por archivo:
- tests/actions/reproduce.rs: `test_reproduce_creates_offspring` (67L) → `test_reproduce_child_in_target_cell`, `test_reproduce_child_inherits_traits`, `test_reproduce_child_born_without_variables` (3 tests). Helper `apply_reproduce` en `actions/common.rs`.
- tests/actions/consume.rs: `test_consume_text_argument_errors` (62L) → `test_consume_text_dx_error`, `test_consume_text_dy_error`, `test_consume_text_damage_error`, `test_consume_text_heal_error` (4 tests).
- tests/actions/move_.rs: `test_move_stops_before_obstacles` (112L) → `test_move_right_two_ends_at_2_0`, `test_move_immediate_block_stays_at_0_0`, `test_move_speed_greater_than_free_path_ends_at_4_0`, `test_move_down_two_ends_at_0_2`, `test_move_up_two_ends_at_2_0` (5 tests).
- tests/sim/determinism.rs: `make_world()` movido a nivel de archivo; `test_simulation_with_injected_rng_is_deterministic` (83L) → `test_injected_rng_same_values_same_history`, `test_injected_rng_says_0_1_2`, `test_injected_rng_different_values_different_history` (3 tests).
- tests/sim/roars.rs: `test_roars_near_and_last_roar` + `test_roar_is_heard_one_tick_only` (≈148L) → `test_roar_not_heard_emission_tick`, `test_roar_heard_next_tick`, `test_roar_no_says_emission_tick`, `test_roar_heard_one_tick_after_emission`, `test_roar_expires_after_one_tick` (5 tests).
- tests/perceptions/nearby.rs: `test_nearest_target_reports_direction` (83L) → `test_nearest_enemy_reports_dist_dx_dy`, `test_nearest_ally_reports_dist_dx_dy`, `test_nearest_object_reports_dist_dx_dy` (3 tests). Helper local `nearest_fixture`.
- tests/perceptions/catalog.rs: `test_catalog_accepts_extra_perception` (106L) → `test_catalog_bonus_absent_and_builtins_present`, `test_catalog_register_bonus_reads_seven` (2 tests). `Bonus`/`EmptyView` movidos a nivel de módulo.
- tests/program/line_clone.rs: `test_line_clone_all_variants` (143L) → `test_line_clone_preserves_label`, `test_line_clone_preserves_goto`, `test_line_clone_preserves_ifgoto`, `test_line_clone_preserves_assign`, `test_line_clone_preserves_action` (5 tests). Helper privado `fixture_program`.

Ficheros modificados:
- crates/instinct-core/tests/actions/common.rs (+helper `apply_reproduce`)
- crates/instinct-core/tests/actions/consume.rs
- crates/instinct-core/tests/actions/move_.rs
- crates/instinct-core/tests/actions/reproduce.rs
- crates/instinct-core/tests/perceptions/catalog.rs
- crates/instinct-core/tests/perceptions/nearby.rs
- crates/instinct-core/tests/program/line_clone.rs
- crates/instinct-core/tests/sim/determinism.rs
- crates/instinct-core/tests/sim/roars.rs

Desviaciones: ninguna. Cero cambios de lógica; aserciones y datos de entrada conservados exactamente. No se tocaron `attack.rs`, `compile/names.rs` ni `interp/errors.rs`.

Puerta: `cargo test --workspace` verde (0 fallos); `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0; árbol limpio tras commit.