---
id: R1-08
title: Splitear pruebas de instinct-cli y deduplicar helpers
phase: R1
status: done
depends_on: [R1-01]
size: M
---

# R1-08 · Splitear pruebas de instinct-cli y deduplicar helpers

## Objetivo

Llevar las tres pruebas planas de `crates/instinct-cli/tests/` al patrón de carpeta que ya usa
`cli_run/`, y quitar la duplicación de helpers entre archivos.

## Alcance

**Haz**
- `tests/cli_annex.rs` (~140) → `tests/cli_annex/{mod.rs, helpers.rs, annex.rs}`.
- `tests/cli_check.rs` (~110) → `tests/cli_check/{mod.rs, helpers.rs, check.rs}`.
- `tests/cli_usage.rs` (~65) → `tests/cli_usage/{mod.rs, helpers.rs, usage.rs}`.
- Deduplica helpers repetidos (`examples_dir`, `instinct_cli_binary`, `run_cli`) contra
  `cli_run/helpers.rs` y `cli_run/e2e_helpers.rs`: extrae a un módulo compartido si el patrón
  actual ya lo permite, o documenta en "Notas de implementación" por qué no (p. ej. helpers
  específicos de escenario). Sin cambiar comportamiento.
- Traduce el comentario en español de `cli_check.rs` (~L58: `Diagnostic form: archivo:línea: mensaje`).
- `tests/cli_usage.rs` L62 usa el mapa inexistente `"inexistente"` como dato de entrada: renómbralo
  a `"missing"` solo si el mensaje de error ya no lo nombra en español tras R1-01; si lo nombra,
  actualiza la aserción afectada en el mismo cambio.

**No hagas**
- No cambies aserciones, códigos de salida ni escenarios.
- No toques `cli_run/` (ya split) salvo para mover un helper compartido con acuerdo de este issue.
- No cambies fixtures `.ins`.

### Archivos previstos
- Los tres archivos planos — mover/dividir.
- `tests/cli_run/helpers.rs`, `tests/cli_run/e2e_helpers.rs` — extraer compartido si aplica.
- `crates/instinct-cli/Cargo.toml` — verificar si el patrón de `[[test]]` lo exige.

## Criterios de aceptación

- [ ] **AC1** Las tres pruebas planas ya no existen; existen las carpetas con `mod.rs` cableado.
  Verif: `git show --stat HEAD`.
- [ ] **AC2** Helpers duplicados eliminados o justificados por escrito. Verif: lectura.
- [ ] **AC3** Mismo número de tests en verde. Verif: `cargo test -p instinct-cli 2>&1 | grep "test result"`.
- [ ] **AC4** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.

## Contexto

`cli_run/` es el patrón de referencia (commit `refactor: split cli_run.rs into tests/cli_run/ submodules`).
Auditoría: `cli_annex` duplica `rows_after_tick` de `e2e_helpers.rs`.

### Leer primero

Los tres archivos planos, `cli_run/mod.rs`, `cli_run/helpers.rs`, `cli_run/e2e_helpers.rs`.

## Notas de implementación

- Rama: `feat/r1-08-split-cli-tests` (checkeada desde `develop`; sin cambios de rama).
- Reparto final: `tests/cli_annex/{mod.rs,helpers.rs,annex.rs}`, `tests/cli_check/{mod.rs,helpers.rs,check.rs}`, `tests/cli_usage/{mod.rs,helpers.rs,usage.rs}` siguiendo el patrón `tests/cli_run/` (`mod.rs` declara submódulos y hace `pub use` de helpers; submódulos importan con `use super::{...}`). Planos borrados: `cli_annex.rs`, `cli_check.rs`, `cli_usage.rs`.
- Decisión sobre helpers compartidos: NO se extrajo a módulo compartido entre targets. Cada `[[test]] target` es un crate separado y `helpers.rs` se mantiene local por carpeta (idéntico a `cli_run` y precedente R1-06); no es posible `use cli_run::helpers` entre targets. Los tres helpers duplicados (`examples_dir`, `instinct_cli_binary`, `run_cli`) se deduplicaron dentro de cada target (una sola definición por target) y `rows_after_tick` vive en `cli_annex/helpers.rs` (idéntico a `cli_run/e2e_helpers.rs`). Justificación por escrito, AC2 cubierta.
- Cambios de firma autorizados: ninguno; todas las firmas de helpers son idénticas a las originales. `fixtures_dir` se mantuvo en `cli_check/helpers.rs` con firma idéntica a la original (uso exclusivo de `check.rs`).
- Archivos modificados: `crates/instinct-cli/Cargo.toml` (3x `[[test]]` añadidos, `cli_run` mantenido); creados los 9 archivos de las 3 carpetas; borrados los 3 planos. Fixtures `.ins` intocables; `cli_run/` sin cambios; `cli_usage`/`cli_run` sin tocar.
- Desviaciones del plan táctico: ninguna. Strings: `"inexistente"` → `"missing"` solo en entrada de `test_cli_unknown_subcommand_exits_2`; aserción de ese test sin tocar. Comentario ES de `cli_check.rs` L58 traducido (`Diagnostic form: file:line: message`); resto de comentarios de `check.rs` traducidos a inglés; aserciones sin cambios.
- Evidencia AC-by-AC:
  - AC1 (PASS): `git status --short` — planos borrados, carpetas con `mod.rs` cableado en `Cargo.toml`.
  - AC2 (PASS): helpers deduplicados por target; decisión documentada arriba.
  - AC3 (PASS): `cargo test -p instinct-cli` → `test result: ok. 2 passed` (annex), `6 passed` (check), `3 passed` (usage); 11 tests CLI, nombres exactos, asserts sin modificar.
  - AC4 (PASS): `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0.
  - Puerta adicional: `cargo test --workspace` exit 0 (0 fallos en todo el workspace).