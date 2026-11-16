---
id: R2-03
title: Auditar errores lógicos remanentes
status: todo
phase: R2
depends_on: [R2-01]
size: M
file: r2-03-logic-errors-audit.md
---

# R2-03 · Auditar errores lógicos remanentes

## Leer primero

Este issue revisa el núcleo en busca de errores lógicos remanentes detectados en la
exploración del proyecto, sin cambiar el comportamiento observable en los caminos
felices. Cada hallazgo se corrige con un test que falle primero (TDD) y luego el código.

## Hallazgos a revisar

### A. `unwrap_or(0)` silencioso en `CreatureSnapshot` (`world/snapshot.rs`)

`health()`, `age()`, `lifespan()`, `x()`, `y()`, `vision()` devuelven `0` cuando la
criatura no existe. Eso puede mascarar bugs (por ejemplo, acceder a una criatura que
murió medio turno). Las percepciones deberían devolver `Option` o al menos no ocultar la
ausencia.

**Decisión:** cambiar las firmas de `CreatureView` para que devuelvan `Option<i64>` /
`Option<&str>` donde corresponda, o dejar `i64` pero documentar que `0` = "no existe".
El issue decide tras leer `perceptions/builtins/` para ver qué consume `0` vs `None`.

### B. Rango de `pc` tras `goto` / `resolve_label` (`interp/lines.rs`)

`jump_target` valida que el destino esté dentro de `program.lines.len()`. Ese check
ya existe. Verificar que no haya otros sitios donde `pc` pueda quedar fuera de rango
sin ser detectado.

### C. Casts de signo a `usize` (`actions/move_.rs`, `actions/parse.rs`)

`move_.rs` usa `pos.x as i64 + dx` y luego `x as usize` tras verificar `is_out_of_bounds`.
Está bien. Verificar que no haya casts silenciosos `as usize` de valores `i64` potencialmente
negativos en otros archivos.

### D. `checked_*` en fin de tick (`world/laws.rs`)

`end_of_tick_creatures` ya usa `checked_sub` / `checked_add`. Verificar que no quede
ninguna aritmética sin checked en `sim/death.rs` u otros módulos de fin de tick.

### E. `diagnostics` público en `World` (`world/world.rs`)

El campo `pub diagnostics: Vec<Diagnostic>` está expuesto. Evaluar si seguirá `pub` o
pasará a `pub(crate)` con getter, para que la GUI no dependa de la representación
interna.

## Criterios de aceptación

1. Cada hallazgo (A–E) tiene un test que falle antes del cambio y pase después.
2. Ningún cambio altera el comportamiento observable en caminos felices (misma semilla
   = misma historia).
3. `cargo test --workspace` → exit 0.
4. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
5. `cargo fmt --all --check` → exit 0.
6. `git status --short` limpio tras commit.

## Notas de implementación

<!-- Rellenar al completar: qué se cambió en cada hallazgo A–E, tests añadidos. -->
