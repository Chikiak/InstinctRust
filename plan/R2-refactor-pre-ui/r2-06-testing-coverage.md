---
id: R2-06
title: Testing coverage gaps
status: done
phase: R2
depends_on: [R2-02, R2-03]
size: M
file: r2-06-testing-coverage.md
---

# R2-06 · Testing coverage gaps

## Leer primero

Este issue cubre huecos de pruebas identificados en la auditoría del proyecto, sin
cambiar lógica de negocio. Cada test nuevo verifica un contrato concreto.

## Huecos a cubrir

1. **`tests/compile/program_tests.rs`** no existe. Añadir pruebas para
   `Program::resolve_label`, `Line::clone`, y que `labels` es un `BTreeMap`
   (orden determinista de iteración).
2. **`Display` de cada variante de `Event`** (`tests/event/display.rs`): verificar
   que todas las variantes (`Said`, `Died`, `RuntimeError`, `Damaged`, `Roared`) se
   formatean correctamente, ya con las strings en inglés de R2-01.
3. **`World::snapshot`** (de R2-02): pruebas que verifiquen que el snapshot refleja
   el estado real tras colocar criaturas, avanzar ticks, y que no muta el mundo.
4. **Edge cases de intérprete:** `IfGoto` verdadero/falso con error, `Goto` a label
   desconocida, error de evaluación de argumentos de acción, `say` múltiple, overflow
   aritmético, `rem` por cero, negación de `i64::MIN`.
5. **Edge cases de mundo:** `damage_cell_content` overflow, `move_creature_to` con ids
   inexistentes, `apply_action_result` con id desconocido, `set_creature_health` /
   `set_variable` con id desconocido.

## Criterios de aceptación

1. Cada hueco listado tiene al menos una prueba que pase.
2. Las pruebas de R2-01 (strings en inglés de eventos) y R2-03 (errores lógicos)
   están incluidas y pasan.
3. `cargo test --workspace` → exit 0.
4. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
5. `cargo fmt --all --check` → exit 0.
6. `git status --short` limpio tras commit.

## Notas de implementación

No se añadió ni duplicó ningún test: los 5 huecos del issue ya estaban cubiertos por tests existentes (ver tabla de evidencia más arriba) y los tests de `tests/interp/` ya existían como archivos; solo faltaba registrar el target de prueba.

- Gap 1 (`Program::resolve_label`, `Line::clone` ×5, `labels` BTreeMap determinista): `tests/program/resolve_label.rs:7-25`, `tests/program/line_clone.rs:41-191`, `tests/compile/labels.rs:50-83` — ejecutados bajo `[[test]] name="program"` y `[[test]] name="compile"`.
- Gap 2 (`Display` de `Event`, strings inglés R2-01): `tests/event/display.rs:7-65` — `[[test]] name="event"`.
- Gap 3 (`World::snapshot`, 7 tests: estado real + no muta): `tests/world/snapshot.rs:7-200` — `[[test]] name="world"`.
- Gap 4 (edge cases de intérprete: IfGoto verdadero/falso/error, Goto a label desconocida, error de argumentos de acción, say múltiple, overflow aritmético, rem por cero, negación de i64::MIN): `tests/interp/if_goto.rs:16-86`, `tests/interp/errors.rs:171-204`, `tests/interp/actions.rs:15-70`, `tests/interp/arithmetic.rs:15-64` — ejecutados tras registrar `[[test]] name = "interp" path = "tests/interp/mod.rs"` en `crates/instinct-core/Cargo.toml` (18 tests: 18 passed).
- Gap 5 (edge cases de mundo): `tests/world/robustness.rs:37-112` — `[[test]] name="world"`.

Cambios en este issue:
- `crates/instinct-core/Cargo.toml`: una línea, `[[test]] name = "interp" path = "tests/interp/mod.rs"` (solo hace que los tests de gap 4 se ejecuten y cuenten para AC1; sin cambio de comportamiento).
- `crates/instinct-core/tests/interp/{common,end_of_program,if_goto,line_limit,turn_flow,wait,errors,arithmetic,runtime_error_bus}.rs`: solo ajustes de import y visibilidad de campo (`pub`) para que el target `interp` compile como suite independiente tras su extracción de `tests/interp_tests.rs`. Sin tocar `src/` y sin cambiar lógica de prueba.

Criterios de aceptación: AC1 ✔ (cada hueco tiene al menos una prueba que pasa), AC2 ✔ (pruebas de R2-01 y R2-03 incluidas y pasando), AC3 ✔ (`cargo test --workspace` → exit 0), AC4 ✔ (`cargo clippy --workspace --all-targets -- -D warnings` → exit 0), AC5 ✔ (`cargo fmt --all --check` → exit 0), AC6 ✔ (`git status --short` limpio tras commit).
