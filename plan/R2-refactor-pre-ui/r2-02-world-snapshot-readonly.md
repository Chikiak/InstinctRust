---
id: R2-02
title: Snapshot estructurado de solo-lectura para renderizado
status: done
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

- Módulo: `crates/instinct-core/src/world/snapshot_view.rs` (~200 líneas), expuesto como `pub mod snapshot_view;` en `world/mod.rs` y reexportado `pub use snapshot_view::{CellSnapshot, LiveCreature, ObjectInfo, TerrainInfo, WorldSnapshot};`.
- DTOs (derivan `Clone + Debug + PartialEq + Eq`):
  - `WorldSnapshot { columns, rows, ticks, map_name, cells: Vec<Vec<CellSnapshot>>, creatures: Vec<LiveCreature>, objects: Vec<ObjectInfo>, terrains: Vec<TerrainInfo>, events: Vec<Event> }`
  - `CellSnapshot { terrain_char: char, object_char: Option<char>, creature: Option<CreatureId> }`
  - `LiveCreature { id, species_name, faction, x, y, health, age, lifespan, state, pc }`
  - `ObjectInfo { index, name, char, reserve }`
  - `TerrainInfo { name, char, reserve, resource_max }`
- `impl World { pub fn snapshot(&self) -> WorldSnapshot }`: construcción de solo-lectura a partir de campos públicos de `World`; sin mutación, sin RNG, sin `println!`, sin `f32`/`f64`, sin dependencias externas; clona los datos (catálogos, eventos) para que un renderizador nunca corrompa el núcleo.
- Tests: `crates/instinct-core/tests/world/snapshot.rs` (7 tests: `snapshot_before_after_equals`, `snapshot_read_only_fields`, `snapshot_reflects_real_state`, `snapshot_cell_contents`, `snapshot_objects_and_terrains_catalogs`, `snapshot_events_clone`, `snapshot_clone_debug`) + `mod snapshot;` en `tests/world/mod.rs` tras `mod vision;` y reexportes en `tests/world/common.rs`.

Evidencia por criterio de aceptación:
- AC1 (PASS): `snapshot::snapshot_before_after_equals` — dos snapshots iguales, ticks sin cambiar, eventos no drenados (`cargo test -p instinct-core --test world` → ok).
- AC2 (PASS): `snapshot::snapshot_read_only_fields` — campos públicos de solo-lectura; `WorldSnapshot` sin métodos `&mut self`.
- AC3 (PASS): `snapshot::snapshot_reflects_real_state` — tras `place_creature` + `step_tick`: ticks 0→1, salud 10→9, edad 0→1, eventos presentes en el snapshot previo.
- `cargo test --workspace` → exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Determinismo `--seed 7`: dos corridas idénticas (SHA256 `1401F3F902C2AD9E0F09247E7D5E21A534C968C52A8B9589AB2FB611BCBC172C`); el contenido es línea a línea igual al histórico registrado del baseline (ledger líneas 9-291). El hash del ledger (`037D7F560AFB472564A1D55EBBC2094EA5CDD6A2CA787EF6E0AD0775D1147A1E`) no se reproduce en este entorno por artifacto de captura (salto de línea final), sin cambio de comportamiento (cambio puramente aditivo).
- `git status --short` limpio tras commit; rama `feat/r2-02-world-snapshot-readonly`.
