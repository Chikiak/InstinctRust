---
id: R1-05
title: `resources/folder.rs` genérico y limpieza de comentarios
phase: R1
status: todo
depends_on: [R1-01]
size: S
---

# R1-05 · `resources/folder.rs` genérico y limpieza de comentarios

## Objetivo

Eliminar la duplicación de los tres cargadores por tipo de recurso en `resources/folder.rs`
(~190 líneas) y quitar comentarios que solo releen el código en `world/snapshot.rs`.

## Alcance

**Haz**
- Unifica los `load_*_folder` en una carga genérica parametrizada por tipo (o helpers compartidos),
  manteniendo exactamente los mismos diagnósticos (texto, línea, archivo) y el mismo orden de
  carga. Si la unificación genérica ensucia más que la duplicación, quédate con helpers comunes y
  dilo en "Notas de implementación".
- `world/snapshot.rs` (~L134-168): borra los comentarios que solo enumeran los códigos de retorno
  ya evidentes del código; deja comentario solo donde aporte el porqué.

**No hagas**
- No cambies reglas de carga (qué archivos se aceptan, extensiones, orden alfabético, diagnóstico
  de archivo inválido).
- No cambies la semilla ni los catálogos.
- No toques `actions/shared.rs` ni `world/laws.rs` (strings de causa: R1-01).

### Archivos previstos
- `crates/instinct-core/src/resources/folder.rs` — modificar.
- `crates/instinct-core/src/world/snapshot.rs` — modificar (comentarios).
- `crates/instinct-core/tests/resources/*` — solo si un import los obliga.

## Criterios de aceptación

- [ ] **AC1** Un solo camino de carga (o helper compartido) sin cambiar diagnósticos. Verif:
  `git show HEAD -- crates/instinct-core/src/resources/folder.rs`.
- [ ] **AC2** `git grep -n "NONE\|OBJECT\|ALLY\|ENEMY\|GROUND" -- crates/instinct-core/src/world/snapshot.rs`
  sigue existiendo en código; los comentarios que los releen ya no. Verif: lectura.
- [ ] **AC3** Suite completa verde. Verif: `cargo test --workspace`.
- [ ] **AC4** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.

## Contexto

Auditoría: tres funciones casi duplicadas por tipo (terreno/objeto/mapa); snapshot con comentarios
redundantes sobre códigos de retorno.

### Leer primero

`crates/instinct-core/src/resources/folder.rs`, `resources/mod.rs`, `world/snapshot.rs`.

## Notas de implementación

（vacío tras completar: enfoque elegido y razón si no fue genérico）