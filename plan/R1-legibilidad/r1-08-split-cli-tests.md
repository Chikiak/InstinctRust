---
id: R1-08
title: Splitear pruebas de instinct-cli y deduplicar helpers
phase: R1
status: todo
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

（vacío tras completar: reparto final y decisión sobre helpers)