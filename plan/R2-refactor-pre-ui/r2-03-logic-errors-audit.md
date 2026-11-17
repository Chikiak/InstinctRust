---
id: R2-03
title: Auditar errores lógicos remanentes
status: done
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

A. unwrap_or(0) en CreatureSnapshot: sin cambio de código. Mantener i64/&str; 0/"" = "creatura no existe" documentado. firmas intactas (snapshot.rs:17-65, view.rs:12-59, builtins/core.rs:15-111 reenvían i64 a Value::Int, no existe Value::Null/Option). TEST AÑADIDO: test_a_creature_snapshot_returns_real_values_not_zero en tests/world/logic_errors_audit.rs (pasa: snapshot refleja health=12, age=0, lifespan=100, x=2, y=1, species_name="Dwarf").

B. pc rango: ya cubierto por tests/interp/errors.rs (test_goto_out_of_range_errors, test_if_goto_out_of_range_errors: goto/if-goto out-of-range -> error, pc no se corrompe). Sin test nuevo (lines.rs:14-36 jump_target valida; turn.rs:54 guarda pc < len).

C. casts as usize tras bounds check: sin cambio (move_.rs:53->56/63; combat.rs:126->129-130; reproduce.rs:65; snapshot.rs:150-151/190-191; parse.rs sin as usize). TEST AÑADIDO: test_c_move_creature_to_out_of_bounds_no_panic en tests/world/logic_errors_audit.rs (pasa: x=3 == columnas devuelve error "outside", criatura no se movió).

D. end_of_tick_creatures usa checked_sub/checked_add; muerte por overflow causa "error" sin pánico (laws.rs:60-116); ya cubre health i64::MIN (robustness.rs:10-35). TEST AÑADIDO: test_d_end_of_tick_age_overflow_no_panic en tests/world/logic_errors_audit.rs (pasa: edad i64::MAX -> checked_add overflow -> Event::Died cause="error", criatura eliminada).

E. diagnostics pub en World (world.rs:48) leído por instinct-cli run.rs:19,76 -> se mantiene pub. WorldSnapshot (snapshot_view.rs) es API GUI separada, sin diagnostics. Sin cambio. TEST AÑADIDO: test_e_diagnostics_public_and_snapshot_separate en tests/world/logic_errors_audit.rs (pasa: diagnostics vacío tras carga limpia, snapshot ticks=0, creatures vacío).

Ningún cambio de código fuente en crates/instinct-core/src/ (cero cambios de comportamiento observable). Tests de integración añadidos en crates/instinct-core/tests/world/logic_errors_audit.rs; módulo registrado en tests/world/mod.rs tras mod robustness.
