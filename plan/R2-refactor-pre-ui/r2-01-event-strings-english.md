---
id: R2-01
title: Strings de eventos al inglés
status: todo
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

<!-- Rellenar al completar: qué strings se renombraron, qué tests se actualizaron. -->
