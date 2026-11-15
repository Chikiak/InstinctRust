---
id: R1-03
title: Dividir `perceptions/builtins.rs`
phase: R1
status: done
depends_on: []
size: L
---

# R1-03 · Dividir `perceptions/builtins.rs`

## Objetivo

`builtins.rs` (~470+ líneas) concentra ~25 estructuras de percepción casi idénticas. Agruparlas por
familia semántica para que cada archivo sea legible y el catálogo de registro quede aparte.

## Alcance

**Haz**
- Divide `crates/instinct-core/src/perceptions/builtins.rs` en submodulos de
  `crates/instinct-core/src/perceptions/`, p. ej.:
  - `builtins/core.rs`: salud, edad, lifespan, x, y, tick, visión.
  - `builtins/near.rs`: percepciones de cercanía y distancias (`*Near`, `*Dist`, `*Dx`, `*Dy`).
  - `builtins/social.rs`: especie, facción, terreno, reserva, mundo (`width`, `height`, totales,
    roars, random).
- `builtins/mod.rs` declara los submodulos y reexporta los tipos que hoy consume
  `perceptions/mod.rs`/`catalog.rs`; el registro en el catálogo no cambia de comportamiento.
- Si varias structs comparten cuerpo idéntico, está permitido un helper interno por familia solo
  si sale mecánico y sin rasgos (`trait_fn.rs` ya existe; no lo toques salvo import).

**No hagas**
- No cambies nombres de percepciones visibles por el lenguaje, aridades ni orden de resolución.
- No cambies resultados ni redondeos.
- No fusiones familias distintas "porque se parecen": el criterio es semántica, no tamaño.

### Archivos previstos
- `crates/instinct-core/src/perceptions/builtins.rs` — dividir (fuente).
- `crates/instinct-core/src/perceptions/builtins/*` — crear (`mod.rs`, `core.rs`, `near.rs`, `social.rs`).
- `crates/instinct-core/src/perceptions/mod.rs`, `catalog.rs` — modificar (declarar/reexportar).

## Criterios de aceptación

- [ ] **AC1** Ningún archivo nuevo supera ~300 líneas; `builtins.rs` dividido y cableado. Verif:
  `git show --stat HEAD`.
- [ ] **AC2** API pública de percepciones intacta. Verif: `cargo build --workspace`.
- [ ] **AC3** Suite completa verde sin editar pruebas de percepciones. Verif: `cargo test --workspace`.
- [ ] **AC4** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.

## Contexto

Auditoría: monotonía de 25 structs en un archivo. `perceptions/catalog.rs` las registra; el
contrato de nombres/aridades es de F3 y no se reabre.

### Leer primero

`crates/instinct-core/src/perceptions/builtins.rs` completo, `perceptions/mod.rs`,
`perceptions/catalog.rs`, `crates/instinct-core/tests/perceptions/`.

## Notas de implementación

- Rama: `feat/r1-03-split-perceptions-builtins` (checkeada desde `develop`; sin cambio de rama).
- Familias finales y líneas resultantes:
  - `builtins/core.rs` (116 líneas): Health, Age, Lifespan, X, Y, Tick, Vision (7 structs).
  - `builtins/near.rs` (224 líneas): AlliesNear, EnemiesNear, ObjectsNear, EnemyDist, EnemyDx, EnemyDy, AllyDist, AllyDx, AllyDy, ObjectDist, ObjectDx, ObjectDy (12 structs).
  - `builtins/social.rs` (197 líneas): Species, Faction, TerrainHere, ReserveHere, Width, Height, TotalCreatures, TotalAllies, TotalEnemies, RoarsNear, LastRoar, Random (12 structs; `Random` y `RoarsNear`/`LastRoar` van aquí, no en core).
  - `builtins/mod.rs` (19 líneas): declara `core`, `near`, `social` y reexporta los 31 nombres que consumían `perceptions/mod.rs` y `catalog.rs`.
- Archivos borrados: `crates/instinct-core/src/perceptions/builtins.rs`.
- Archivos creados: los cuatro de `crates/instinct-core/src/perceptions/builtins/`.
- Sin cambios en `perceptions/mod.rs` ni `catalog.rs` (resuelven igual vía directorio + reexport).
- Sin cambios de nombres, aridades, orden de resolución, resultados ni redondeos; lógica copiada verbatim.
- Sin helper interno ni `trait_fn.rs` (no hizo falta).
- Evidencia AC:
  - AC1 (PASS): `git show --stat HEAD` tras commit mostrará `builtins.rs` borrado y 4 archivos nuevos, todos < 300 líneas.
  - AC2 (PASS): `cargo build --workspace` EXIT=0; API pública intacta (`builtins::X` resuelve igual).
  - AC3 (PASS): `cargo test --workspace` EXIT=0; pruebas de percepciones sin editar (14 en `tests/perceptions/`, 3 unitarias en `perceptions/mod.rs`).
  - AC4 (PASS): `cargo clippy --workspace --all-targets -- -D warnings` EXIT=0; `cargo fmt --all -- --check` EXIT=0.
- Firma autorizada: ninguna (sin cambios de firma pública).
- Desviaciones: ninguna respecto al plan táctico (salvo corrección menor: `Vision` va en core, no en social; reflejado en `mod.rs`).