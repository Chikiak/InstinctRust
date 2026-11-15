---
id: R1-06
title: Splitear pruebas planas de instinct-core
phase: R1
status: todo
depends_on: [R1-01]
size: M
---

# R1-06 · Splitear pruebas planas de instinct-core

## Objetivo

Cinco archivos de prueba siguen planos bajo `crates/instinct-core/tests/` sin el patrón de carpeta
que ya usa el resto (`tests/<área>/{mod.rs, common.rs, ...}`). Llevarlos al patrón y traducir sus
comentarios en español.

## Alcance

**Haz** (mover archivos, no reescribir pruebas: byte a byte salvo comentarios)
- `tests/rng_tests.rs` → `tests/rng/{mod.rs, common.rs, gen.rs, world.rs}` (reparto por cohesión).
- `tests/event_tests.rs` → `tests/event/{mod.rs, common.rs, shared.rs, display.rs, world_output.rs}`.
- `tests/value_tests.rs` → `tests/value/{mod.rs, common.rs, ops.rs, truthy.rs}`.
- `tests/species_tests.rs` → `tests/species/{mod.rs, common.rs}`.
- `tests/diagnostic_tests.rs`: 13 líneas, una prueba; dividir solo si encaja sin forzar
  (`tests/diagnostic/{mod.rs, display.rs}`); si no, se queda plano y se anota.
- Cada `mod.rs` usa el patrón existente: `mod common; mod <sub>; …` + `pub use common::*;` cuando
  aplique (ver `tests/world/mod.rs`, `tests/actions/mod.rs`).
- Traduce a inglés conciso los comentarios en español citados en la auditoría:
  `event_tests.rs` L5-6, 64-65, 128-129, 156, 169, 181; `value_tests.rs` L93;
  `species_tests.rs` L23-25, 44-45, 52-53, 59. Borra los redundantes en lugar de traducirlos.

**No hagas**
- No cambies ni una aserción, ni datos de entrada, ni nombres de test (salvo que el módulo nuevo
  lo requiera y siga siendo el mismo test).
- No toques las pruebas de `cli_run/`, `rng` de `instinct-cli`, ni otras carpetas.
- No fusiones `common.rs` de carpetas distintas (auditoría: los 9 `common.rs` tienen contenido
  distinto, no están duplicados).

### Archivos previstos
- Los cinco archivos planos anteriores — mover/dividir.
- `crates/instinct-core/tests/**` — posibles imports por mudanza.
- `crates/instinct-core/Cargo.toml` — normalmente no cambia (los `[[test]]` son por archivo
  `mod.rs` si el patrón actual lo hace; verifícalo y documéntalo).

## Criterios de aceptación

- [ ] **AC1** Los archivos planos mencionados ya no existen como tales; existen las carpetas con
  `mod.rs` cableado. Verif: `git show --stat HEAD`.
- [ ] **AC2** `git grep -n -iE 'comparte|puerto|variante|guard|recibi|orden|negativos' -- crates/instinct-core/tests`
  sin coincidencias en comentarios traducidos. Verif: comando.
- [ ] **AC3** Mismo número de tests en verde que antes (contar con `cargo test --workspace 2>&1 | grep
  "test result"`). Verif: comparación.
- [ ] **AC4** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.

## Contexto

Patrón ya aplicado en `tests/lexer`, `tests/program`, `tests/actions`, `tests/cli_run` de la CLI
(ver git log). Los `common.rs` por carpeta son distintos entre sí: no hay deduplicación pendiente.

### Leer primero

Los cinco archivos planos, `tests/world/mod.rs` y `tests/actions/mod.rs` como patrón, y un
ejemplo ya split (`tests/lexer/mod.rs`).

## Notas de implementación

（vacío tras completar: reparto final y número de tests antes/después）