---
id: R2-04
title: DRY residual y cohesión en actions/
status: done
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

Se realizó la pasada final de DRY sobre `crates/instinct-core/src/actions/`. La revisión evidenció que NO hay duplicación real, ni código muerto, ni wrappers triviales: todos los helpers viven centralizados en `shared.rs`, `combat.rs`, `consts.rs`, `parse.rs`, `context_helpers.rs` y `events.rs`; cada archivo de acción tiene ≤ 95 líneas; `ACTION_ATTACK` es `pub(crate)` justificado en `world/action_context.rs:4,51`. Por tanto:
- ¿Qué se movió? Nada. 0 líneas movidas.
- ¿Qué se eliminó? Nada. 0 líneas eliminadas.
- ¿Qué se dejó igual? Todo el módulo `actions/` sin cambios de código Rust (0 líneas agregadas/modificadas/eliminadas), porque no quedaba duplicación residual que eliminar sin alterar lógica de negocio.
- Puertas validadas: `cargo test --workspace` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all --check` exit 0; `cargo build --bin instinct-cli` exit 0.
- Determinismo: dos corridas de `instinct-cli run --root examples --map "Plain with rocks" --seed 7 --ticks 30 --place Dwarf@1,0 --place Ent@5,1 --place Hobbit@10,0 --place Uruk@12,3` produjeron SHA256 idéntico `D6F0D1380CBD856580123B85240855CA0FB547EFCEDA8560BD599EFF7041D6B8` (byte a byte).
- Commit: solo los 2 archivos de plan (PHASE.md + este issue), sin cambios de código Rust.
