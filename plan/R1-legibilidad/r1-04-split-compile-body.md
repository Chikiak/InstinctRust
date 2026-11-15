---
id: R1-04
title: Dividir `compile/body.rs`
phase: R1
status: done
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

Rama: `feat/r1-04-split-compile-body` (commit sobre `develop`).

Reparto final de fases en `compile/`:
- `crates/instinct-core/src/compile/body.rs` (143 líneas): `LabelRef` (`pub`, no reexportado), `CompileState` (diagnostics, lines, labels, label_refs, found_start, seen_keys, species_name, faction, health, vision, lifespan, header) y `compile()` como orquestador: `CompileState::new()` → bucle línea a línea (`strip_trailing_comment` se queda en el bucle) → `process_header_line` (cabecera, antes de `start:`) → `dispatch_body_line` (cuerpo) → `finalize_validation` → `CompileResult`. Firma de `compile()` intacta; `diagnostics.sort_by_key(|d| d.line)` al final; textos de diagnóstico idénticos.
- `crates/instinct-core/src/compile/header.rs` (178 líneas): `Header`, `HEADER_KEYS`, `split_header`, `validate_header_value` y `process_header_line` (movido verbatim de `body.rs` líneas 74–174, adaptado a `CompileState`; retorna `true` en `start:` insertando `labels.insert("start", ...)` y `found_start = true`).
- `crates\instinct-core\src\compile\dispatch.rs` (nuevo, 138 líneas): `dispatch_body_line` movido verbatim de `body.rs` líneas 180–294 (label, assignment, goto, if, action call, unrecognized), ahora recibe `&mut CompileState` (7 argumentos, sin `too_many_arguments`).
- `crates\instinct-core\src\compile\validate.rs` (nuevo, 64 líneas): `finalize_validation` movido verbatim del bloque de post-validación de `body.rs` (líneas 200–246 pre-refactor): claves requeridas en línea 1, montaje de `Header`, `goto` a etiqueta desconocida, etiqueta `start` requerida y ordenado de diagnósticos.
- `crates\instinct-core\src\compile\mod.rs`: `mod validate;` añadido; `pub use body::compile; pub use header::Header;` sin cambios.

Puerta de salida: `cargo test --workspace` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all -- --check` exit 0; `git status` limpio tras el commit. Ningún cambio de lógica ni de orden/texto de diagnósticos.