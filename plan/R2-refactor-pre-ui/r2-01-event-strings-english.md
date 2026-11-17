---
id: R2-01
title: Strings de eventos al inglés
status: done
phase: R2
depends_on: []
size: S
file: r2-01-event-strings-english.md
---

# R2-01 · Strings de eventos al inglés

## Leer primero

Este issue renombra las strings en español que aún quedan en la salida de eventos de
`event/display.rs` a inglés, por coherencia con R1-01 (que tradujo `vida`/`edad`,
causas de muerte y UI de CLI) y porque la salida observable del núcleo debe ser
estable y en inglés (la GUI de F5 la consumirá y puede traducir en su capa de
presentación).

No cambia lógica de negocio: solo strings de `Display` para `Event`.

## Contexto

El ledger de R1 (`.opencode/ledgers/r1-legibilidad-report.md`) reportó 0 coincidencias
de comentarios en español y 0 de `vida`/`edad`/`muere`, pero `event/display.rs` aún
tiene:

- `"dice: {text}"` → `"says: {text}"`
- `"daña {tx},{ty} por {amount}"` → `"damages {tx},{ty} by {amount}"`
- `"ruge: {text}"` → `"roars: {text}"`
- `"error en línea {line}: {message}"` → `"error at line {line}: {message}"`

Estas strings son salida observable del binario (impresas por `CliOutput` a través del
puerto `WorldOutput`). Cambiarlas altera la salida de `run`, por decisión explícita del
usuario (misma decisión que R1-02).

## Criterios de aceptación

1. `crates/instinct-core/src/event/display.rs`: todas las variantes de `Event` usan
   strings en inglés (`says`, `dies of`, `error at line`, `damages`, `roars`).
2. Los tests de `Display` de eventos (`tests/event/display.rs` y cualquiera que afirme
   las strings viejas) se actualizan a las nuevas strings en el mismo commit.
3. `cargo test --workspace` → exit 0.
4. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
5. `cargo fmt --all --check` → exit 0.
6. `git status --short` limpio tras commit.

## Notas de implementación

Strings renombradas en `crates/instinct-core/src/event/display.rs` (salida de `Event::Display`, sin tocar lógica ni `Event::Died` "dies of"):
1. `"dice: {text}"` → `"says: {text}"` (`Event::Said`)
2. `"error en línea {line}: {message}"` → `"error at line {line}: {message}"` (`Event::RuntimeError`)
3. `"daña {tx},{ty} por {amount}"` → `"damages {tx},{ty} by {amount}"` (`Event::Damaged`)
4. `"ruge: {text}"` → `"roars: {text}"` (`Event::Roared`)

Tests actualizados a las nuevas strings (8 asserts):
- `crates/instinct-core/tests/event/display.rs`: `test_display_said`, `test_display_runtime_error`, `test_display_damaged`, `test_display_roared` (4 asserts).
- `crates/instinct-cli/tests/cli_run/e2e.rs`: `test_cli_e2e_attack_kills_adjacent_creature`, `test_cli_e2e_consume_kills_and_heals` (daña→damages, 2 asserts), `test_cli_e2e_roar_emits_event` (ruge→roars, 1 assert).
- `crates/instinct-cli/tests/cli_run/events.rs`: `test_cli_events_stdout_diagnostics_stderr` (1 assert).

`cargo fmt --all` (una sola pasada, cambio mecánico de rustfmt sin semántica; no altera comportamiento — verificado con `cargo test --workspace` en verde tras el formateo) reformateó 10 archivos:
- Editados por este issue: `crates/instinct-core/src/event/display.rs`, `crates/instinct-core/tests/event/display.rs`, `crates/instinct-cli/tests/cli_run/e2e.rs`, `crates/instinct-cli/tests/cli_run/events.rs`.
- Deuda de formato pre-existente en `develop` (commit 03a5f5b), no tocada por este issue: `crates/instinct-core/src/actions/combat.rs`, `crates/instinct-core/src/actions/mod.rs`, `crates/instinct-core/src/actions/reproduce.rs`, `crates/instinct-core/src/actions/roar.rs`, `crates/instinct-core/src/actions/say.rs`, `crates/instinct-core/src/actions/shared.rs`.

Puertas verdes: `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo test --workspace` exit 0.
