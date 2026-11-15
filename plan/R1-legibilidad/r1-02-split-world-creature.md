---
id: R1-02
title: Dividir `world/creature.rs`
phase: R1
status: done
depends_on: [R1-01]
size: L
---

# R1-02 · Dividir `world/creature.rs`

## Objetivo

`creature.rs` (~530 líneas) mezcla tipos de criatura con el mundo completo (carga de mapa,
colocación, RNG, eventos, aplicación de resultados de acción). Separar por responsabilidad para
que cada archivo quede por debajo de ~300 líneas y con una sola razón de cambiar.

## Alcance

**Haz**
- Divide en submodulos dentro de `crates/instinct-core/src/world/` conservando la API pública
  (`world/mod.rs` reexporta lo que hoy exporta; ningún consumidor cambia salvo imports):
  - `creature/types.rs` (o `creature.rs` residual): tipos de criatura + Memento relacionado.
  - `world.rs` (o `world/world.rs`): struct `World`, `from_map`/constructores, getters de rejilla.
  - `life.rs`: colocar/eliminar/mover criaturas, salud, muertes.
  - `ticks.rs`: RNG del mundo, emisión/cola de eventos, `apply_action_result`, leyes de fin de tick
    si hoy viven aquí.
- El reparto exacto lo decide el coordinador por cohesión; la restricción es: cada archivo con una
  responsabilidad, `visibility` mínima necesaria, sin cambiar comportamiento ni firmas públicas.
- Mantén los tests existentes sin cambios (son la red); si un test necesita un import nuevo por la
  mudanza, es solo import.

**No hagas**
- No cambies semilla, orden de turnos, fórmulas de daño/curación ni eventos emitidos.
- No inventes traits ni capas nuevas: es mudanza de código, no rediseño.
- No dejes código muerto ni `pub use` redundantes: clippy `-D warnings` debe pasar.

### Archivos previstos
- `crates/instinct-core/src/world/creature.rs` — dividir (fuente).
- `crates/instinct-core/src/world/mod.rs` — modificar (declarar submodulos, reexportar).
- Otros módulos de `src/` — modificar solo imports si la mudanza los obliga.

## Criterios de aceptación

- [ ] **AC1** `world/creature.rs` ya no existe o queda <300 líneas con una sola responsabilidad; los
  nuevos archivos existen y están cableados en `world/mod.rs`. Verif: `git show --stat HEAD`.
- [ ] **AC2** API pública intacta: `cargo build --workspace` y la CLI compilan sin cambios de
  llamada salvo imports. Verif: `cargo build -p instinct-cli`.
- [ ] **AC3** Suite completa verde sin editar pruebas. Verif: `cargo test --workspace`.
- [ ] **AC4** Clippy/fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.
- [ ] **AC5** Determinismo: pruebas con semilla fija pasan sin modificación. Verif:
  `cargo test -p instinct-core --test annex_examples determinism`.

## Contexto

Auditoría (`.opencode/ledgers/refactor-recon-report.md`): el archivo contiene tipos `Creature` y el
`impl World` con mapa, colocación, RNG, eventos y `apply_action_result`. `world/mod.rs` ya existe y
exporta los submódulos actuales (`grid`, `laws`, `snapshot`, etc.); el split se engancha a ese
patrón.

### Leer primero

`crates/instinct-core/src/world/creature.rs` completo, `world/mod.rs`, y los consumidores
principales (`sim/`, `interp/`, `actions/`, `crates/instinct-cli/src/`).

## Notas de implementación

Rama: `feat/r1-02-split-world-creature` (checkeada desde `develop`, sin saltos de rama).

Reparto final por archivo y líneas resultantes:
- `crates/instinct-core/src/world/creature/types.rs` (81 líneas): `CreatureId`, `CellContent`, `CreatureState`, `Creature`, `CreatureMemento` + `impl Creature::memento`. Copia verbatim de `creature.rs` l.15-95. Imports: `crate::species::Species`, `crate::value::Value`, `std::collections::HashMap`, `std::rc::Rc`.
- `crates/instinct-core/src/world/creature/mod.rs` (5 líneas): declara `pub mod types;` y reexporta `CellContent, Creature, CreatureId, CreatureMemento, CreatureState`. Sin `World`.
- `crates/instinct-core/src/world/world.rs` (154 líneas): `pub struct World` (todos los campos `pub`) + `impl World { pub fn from_map(...) }`. Copia verbatim de `creature.rs` l.101-244. Sin lógica de ciclo de vida ni ticks.
- `crates/instinct-core/src/world/life.rs` (208 líneas): `impl World` con `place_creature`, `remove_creature`, `live_creature_ids`, `creature_position`, `creature_species`, `creature` (pub(crate)), `creature_mut` (pub(crate)), `variable`, `set_creature_health`, `set_variable`, `restore_creature_from_memento`, `move_creature_to`. Copia verbatim de `creature.rs`. Solo ciclo de vida de criaturas.
- `crates/instinct-core/src/world/ticks.rs` (75 líneas): `impl World` con `set_rng_seed`, `set_rng_source`, `set_output`, `tick`, `advance_tick`, `emit_event`, `drain_events`, `apply_action_result`. Copia verbatim de `creature.rs`. Solo temporización/eventos/RNG/output.
- `crates/instinct-core/src/world/creature.rs`: borrado (Paso 7).
- `crates/instinct-core/src/world/mod.rs`: añade `pub mod life;`, `pub mod ticks;`, `pub mod world;`; cambia `pub use creature::{..., World}` a `pub use creature::{...}` + `pub use world::World;`. Conjunto público idéntico (6 nombres).
- `crates/instinct-core/src/world/clone_debug.rs`: `use super::creature::World;` → `use super::World;`. Cuerpo de Clone/Debug sin tocar.

AC-by-AC evidence:
- AC1 (PASS): `git show --stat` confirma `creature.rs` eliminado; `creature/`, `world.rs`, `life.rs`, `ticks.rs` existen y están cableados en `world/mod.rs`.
- AC2 (PASS): `cargo build -p instinct-cli` → exit 0. API pública intacta (`crate::world::{CellContent, Creature, CreatureId, CreatureMemento, CreatureState, World}`).
- AC3 (PASS): `cargo test --workspace` → todo verde, 0 fallos.
- AC4 (PASS): `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all -- --check` → exit 0.
- AC5 (PASS): `cargo test -p instinct-core --test annex_examples determinism` → exit 0.

Cambios de firma autorizados: ninguno (cero cambios de lógica, cero cambios de firma pública).
Archivos modificados: `clone_debug.rs`, `mod.rs`, `creature.rs` (borrado), `creature/types.rs` (nuevo), `creature/mod.rs` (nuevo), `world.rs` (nuevo), `life.rs` (nuevo), `ticks.rs` (nuevo), issue R1-02, PHASE.md.
Desviaciones: ninguna. Se añadió `#[allow(clippy::module_inception)]` en `pub mod world;` dentro de `world/mod.rs` para satisfacer clippy `-D warnings` manteniendo el nombre de módulo exigido por el plan táctico.