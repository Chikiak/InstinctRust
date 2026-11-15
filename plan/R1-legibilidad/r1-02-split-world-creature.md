---
id: R1-02
title: Dividir `world/creature.rs`
phase: R1
status: todo
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

（vacío tras completar: reparto final por archivo y líneas resultantes）