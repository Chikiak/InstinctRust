---
id: R1-04
title: Dividir `compile/body.rs`
phase: R1
status: todo
depends_on: []
size: M
---

# R1-04 · Dividir `compile/body.rs`

## Objetivo

`compile/body.rs` (~330 líneas) mezcla la fase de cabecera, el despacho del cuerpo y la
post-validación dentro de `compile()`. Separar fases para que el flujo de compilación se lea de
un vistazo.

## Alcance

**Haz**
- Mueve la fase de cabecera a `crates/instinct-core/src/compile/header.rs` (ya existe; consolidar
  allí lo que de cabecera vive en `body.rs`).
- Deja en `body.rs` (o `body/dispatch.rs`) solo el despacho por línea de cuerpo.
- Si la post-validación tiene identidad propia, extraerla a su propia función/archivo con nombre
  que la describa.
- `compile/mod.rs` reexporta la API actual; `compile()` sigue con la misma firma y el mismo orden
  de diagnósticos.

**No hagas**
- No cambies el orden ni el texto de los diagnósticos.
- No cambies qué errores abortan la compilación y cuáos se acumulan.
- No toques el lexer ni el parser de expresiones.

### Archivos previstos
- `crates/instinct-core/src/compile/body.rs` — dividir.
- `crates/instinct-core/src/compile/header.rs` — modificar (absorber fase de cabecera).
- `crates/instinct-core/src/compile/mod.rs` — modificar (cableado).
- `crates/instinct-core/tests/compile/*` — solo si un import los obliga.

## Criterios de aceptación

- [ ] **AC1** `body.rs` queda <300 líneas y con una responsabilidad; fases separadas y cableadas.
  Verif: `git show --stat HEAD`.
- [ ] **AC2** API de compilación intacta. Verif: `cargo build --workspace`.
- [ ] **AC3** Suite completa verde. Verif: `cargo test --workspace`.
- [ ] **AC4** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.

## Contexto

`compile/` ya está modularizado (`action.rs`, `assignment.rs`, `comment.rs`, `header.rs`); este
issue sigue el mismo patrón.

### Leer primero

`crates/instinct-core/src/compile/body.rs` completo, `compile/mod.rs`, `compile/header.rs`.

## Notas de implementación

（vacío tras completar: reparto final y líneas resultantes）