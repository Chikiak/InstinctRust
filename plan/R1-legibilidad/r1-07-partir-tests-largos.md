---
id: R1-07
title: Partir tests largos en módulos ya-split
phase: R1
status: todo
depends_on: [R1-06]
size: L
---

# R1-07 · Partir tests largos en módulos ya-split

## Objetivo

Ocho módulos de prueba ya viven en carpetas, pero contienen tests de 60-140 líneas que apilan
escenarios en una sola función. Partirlos en tests enfocados y helpers compartidos, sin cambiar
una sola aserción.

## Alcance

**Haz** (mover/splitear; los tests resultantes deben afirmar exactamente lo mismo, repartido)
- `tests/actions/reproduce.rs` (~290): el test de ~67 líneas (L20-87) → varios tests o helpers.
- `tests/actions/consume.rs` (~220): el test de ~61 líneas (L173-234) → dividir.
- `tests/actions/move_.rs` (~190): el test de ~112 líneas (L9-121) → dividir.
- `tests/sim/determinism.rs` (~215): el test de ~82 líneas (L142-224) → dividir.
- `tests/sim/roars.rs` (~195): tests de ~66 y ~82 líneas (L54-120, L121-203) → dividir.
- `tests/perceptions/nearby.rs` (~190): test de ~83 líneas (L60-143) → dividir.
- `tests/perceptions/catalog.rs` (~185): test de ~106 líneas (L8-114) → dividir.
- `tests/program/line_clone.rs` (~165): test único de ~142 líneas → varios tests/helpers.
- Helpers comunes vía `common.rs` de cada carpeta (ya existen; no los cruces entre carpetas).

**No hagas**
- No cambies datos de entrada ni expectativas; si al dividir parece que sobra una aserción,
  se mantiene.
- No cambies nombres de tests salvo que el split lo exija; si renombras, que el nombre siga
  describiendo el mismo comportamiento.
- No toques `attack.rs`, `compile/names.rs`, `interp/errors.rs` (ya tienen tests cortos).

### Archivos previstos
- Los ocho archivos anteriores — modificar (dividir tests, añadir helpers).
- `common.rs` de las carpetas afectadas — modificar si hace falta un helper compartido.

## Criterios de aceptación

- [ ] **AC1** Ningún archivo de prueba de la lista queda con un test de más de ~60 líneas. Verif:
  lectura + `git show --stat HEAD`.
- [ ] **AC2** Mismo número de tests en verde que antes o más (nunca menos aserciones). Verif:
  `cargo test --workspace 2>&1 | grep "test result"` y comparación de totales.
- [ ] **AC3** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.

## Contexto

Auditoría: los archivos ya están split; la deuda son tests gigantes, no estructura de archivos.

### Leer primero

Los ocho archivos, sus `mod.rs` y `common.rs`.

## Notas de implementación

（vacío tras completar: tests resultantes por archivo)