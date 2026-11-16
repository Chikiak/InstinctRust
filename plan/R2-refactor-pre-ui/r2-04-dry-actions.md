---
id: R2-04
title: DRY residual y cohesión en actions/
status: todo
phase: R2
depends_on: [R2-01]
size: M
file: r2-04-dry-actions.md
---

# R2-04 · DRY residual y cohesión en actions/

## Leer primero

El módulo `actions/` ya fue muy refactorizado en fases previas (actions-cleanup,
actions-module-improvements): helpers compartidos en `context_helpers.rs`, `parse.rs`,
`shared.rs`, `events.rs`, `combat.rs`, `consts.rs`; `say`/`roar` unificadas vía
`say_like`; `attack`/`consume` vía `apply_strike` y `parse_strike_args`.

Este issue hace una pasada final para eliminar cualquier duplicación residual y mejorar
la cohesión, sin cambiar lógica de negocio.

## Qué revisar

1. **`combat.rs` vs `shared.rs`:** `apply_strike`, `heal_attacker`, `strike_*` están en
   `combat.rs` y se re-exportan desde `shared.rs`. Verificar que no haya wrappers triviales
   duplicados.
2. **`consts.rs`:** las constantes `ACTION_*`, `ARG_*`, `DIAG_*` están bien centralizadas.
   Verificar que no haya strings mágicas duplicadas en los archivos de acción.
3. **`attack.rs`, `consume.rs`, `move_.rs`, `wait.rs`, `roar.rs`, `say.rs`,
   `reproduce.rs`:** cada uno es corto (< 100 líneas) y usa helpers. Verificar que no haya
   lógica que pudiera subirse a `shared.rs` o `context_helpers.rs`.
4. **`parse.rs`:** `check_arity`, `parse_int_arg`, `parse_dir_arg`, `parse_dir_args`,
   `parse_strike_args` están bien. Verificar que no haya parseo de argumentos duplicado
   en los archivos de acción.
5. **Visibilidad:** `pub(super) use` en `shared.rs` re-exporta todo. Verificar que no haya
   imports innecesarios ni `#[allow(unused_imports)]` que cubran código muerto.

## Criterios de aceptación

1. No hay duplicación de lógica entre los archivos de acción y los helpers compartidos.
2. Todos los archivos de acción siguen siendo ≤ ~100 líneas (salvo `reproduce.rs` que es
   más compleja y está alrededor de 94).
3. `cargo test --workspace` → exit 0.
4. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
5. `cargo fmt --all --check` → exit 0.
6. `git status --short` limpio tras commit.

## Notas de implementación

<!-- Rellenar al completar: qué se movió, qué se eliminó, qué se dejó igual y por qué. -->
