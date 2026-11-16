---
id: R2-02
title: Snapshot estructurado de solo-lectura para renderizado
status: todo
phase: R2
depends_on: [R2-01]
size: M
file: r2-02-world-snapshot-readonly.md
---

# R2-02 · Snapshot estructurado de solo-lectura para renderizado

## Leer primero

Este issue crea una API de solo-lectura que expone el estado del mundo de forma
estructurada, lista para que la ventana (F5) la consuma sin mutar el núcleo y sin
cambios de API en el núcleo.

Hoy `World::draw()` devuelve un `String` ASCII, útil para la CLI pero insuficiente para
una GUI que necesita saber, por cada celda: qué criatura está ahí (id, especie, salud,
edad), qué objeto, qué terreno, y los eventos recientes.

## Contexto

- `crates/instinct-core/src/world/draw.rs` dibuja ASCII y valida índices contra los
  catálogos propios del mundo.
- `crates/instinct-core/src/world/snapshot.rs` tiene `CreatureSnapshot` para percepciones
  (solo lectura, salvo `random`).
- `crates/instinct-core/src/world/world.rs` tiene campos públicos (`creatures`,
  `terrains`, `objects`, `content`, `events`, etc.) que una GUI podría leer directamente,
  pero eso acopla la GUI a la representación interna.

## Qué entregar

Un tipo `WorldSnapshot` (o similar) en `crates/instinct-core/src/world/snapshot.rs` (o
nuevo módulo `world/snapshot_view.rs`) que ofrezca, de forma inmutable:

- `columns`, `rows`, `ticks`, `map_name`
- Para cada celda `(x, y)`: carácter de terreno, carácter de objeto (si hay),
  `Option<CreatureId>` de criatura
- Para cada criatura viva: id, nombre de especie, facción, posición (x, y), salud, edad,
  lifespan, estado (`Awake`/`Sleeping(n)`), pc
- Para cada objeto: índice, nombre, carácter, reserva
- Para cada terreno: nombre, carácter, reserva, resource_max
- Eventos recientes (desde el último `drain_events` o una copia del vec de eventos)
- Método `World::snapshot(&self) -> WorldSnapshot` que construye el DTO sin mutar nada

El tipo debe ser `Clone` y `Debug` para facilitar pruebas y depuración en la GUI.

## Criterios de aceptación

1. `World::snapshot(&self) -> WorldSnapshot` existe y no muta `self`.
2. `WorldSnapshot` expide los datos listados arriba de forma de solo-lectura (campos
   públicos o getters, sin `&mut self` ni métodos que muten el mundo).
3. `cargo test --workspace` → exit 0 (incluir tests que verifiquen que el snapshot
   refleja el estado real del mundo: colocar criatura, avanzar tick, comparar snapshot).
4. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
5. `cargo fmt --all --check` → exit 0.
6. `git status --short` limpio tras commit.
7. Determinismo intacto: `--seed 7` sobre los 4 ejemplos del anexo da misma historia.

## Notas de implementación

<!-- Rellenar al completar: módulo elegido, campos de WorldSnapshot, tests añadidos. -->
