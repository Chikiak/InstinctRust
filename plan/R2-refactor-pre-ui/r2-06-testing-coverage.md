---
id: R2-06
title: Testing coverage gaps
status: todo
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

<!-- Rellenar al completar: qué tests se añadieron, en qué archivos. -->
