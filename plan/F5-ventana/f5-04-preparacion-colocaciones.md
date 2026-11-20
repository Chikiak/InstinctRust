---
id: F5-04
title: Preparación - elegir mapa, colocar y quitar criaturas
status: done
phase: F5-ventana
depends_on: [F5-03]
size: M
file: f5-04-preparacion-colocaciones.md
---

# F5-04 · Preparación: colocaciones sobre la máquina de estados (4.1 paso 2)

## Leer primero

- `crates/instinct-core/src/world/world.rs` (`World::from_map` L71) y `world/life.rs`
  (`place_creature` L20-42 valida bounds y celda ocupada; `remove_creature` L80-85;
  `live_creature_ids` L91; `creature_position` L101)
- `crates/instinct-core/src/world/grid.rs` (dimensiones del mapa y celdas)
- `plan/F5-ventana/f5-03-carga-catalogos.md` (la sesión cargada)
- Enunciado §4.1 paso 2 (`docs/enunciado.md:384`): colocar varias de la misma especie,
  quitar las colocadas por error, nunca sobre objeto ni otra criatura.

## Tarea

1. Estado de preparación (`app/prep.rs`), headless y sin eframe:
   - Receta de la simulación: `map_name`, `seed`, colocaciones
     `Vec<(species_name, x, y)>`. La GUI guarda la **receta**, no el mundo (decisión 4:
     el reinicio recrea desde la receta).
   - `enter_prep(session, map_name) -> Result<PrepState, Diagnostic>`: construye el
     mundo con `World::from_map` y deja colocaciones vacías.
   - `place(species, x, y) -> Result<PlacementId, Diagnostic>`: valida especie
     conocida, bounds y celda libre (delegando en el mundo), y guarda la colocación en
     la receta. Varias colocaciones de la misma especie están permitidas.
   - `remove(placement_id) -> Result<(), Diagnostic>`: quita la criatura del mundo y de
     la receta (solo en preparación).
   - `occupied_cell(x, y)` y `creatures_in_prep()` para que la UI pinte colocaciones.
2. Reglas headless testeables:
   - No se puede colocar sobre objeto ni criatura ocupada (error con `Diagnostic`,
     la app sigue viva).
   - Fuera de bounds: error, no pánico.
   - Quitar una colocada por error deja la celda libre otra vez.
   - Colocar N criaturas de la misma especie en celdas libres funciona; el conteo de
     criaturas vivas del mundo lo refleja.
3. Capa `ui/` (eframe): panel de preparación con selector de especie, rejilla clicable
   para colocar (clic en celda libre coloca la especie elegida; clic en criatura
   colocada la quita, con confirmación visual en el registro) y lista de colocadas.
   La UI solo envía mensajes; todas las reglas viven en `app/prep.rs`.

## Criterios de aceptación

1. Tests headless: colocación válida; colocación sobre objeto/criatura y fuera de
   bounds devuelve `Diagnostic` sin pánico; quitar una colocada; N colocaciones de la
   misma especie; el mundo queda consistente (celda libre tras quitar).
2. La receta resultante (mapa, semilla, colocaciones) es suficiente para recrear la
   simulación idéntica (lo prueba F5-08).
3. `cargo test --workspace` exit 0; clippy/fmt verdes.
4. Cero cambios en `crates/instinct-core/`; archivos < ~300 líneas; `git status` limpio.

## Verificación

```powershell
cargo test -p instinct-gui
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p instinct-gui   # manual: colocar y quitar criaturas
```

## Notas de implementación

- Rama: feat/f5-04-preparacion-colocaciones (base develop).
- Archivos creados: crates/instinct-gui/src/app/prep.rs (~130 líneas, headless, sin eframe); crates/instinct-gui/tests/prep/common.rs; crates/instinct-gui/tests/prep/mod.rs (11 tests t-prep-01..t-prep-11).
- Archivos modificados: crates/instinct-gui/Cargo.toml ([[test]] prep); crates/instinct-gui/src/app/mod.rs (pub mod prep; prep_state/prep_selected_species en AppState; 4 Msg variantes PrepEnter/PrepPlace/PrepRemove/PrepSelectSpecies + brazos en update); crates/instinct-gui/src/app/session.rs (accesores resources(), selected_map_index()); crates/instinct-gui/src/main.rs (inicializa nuevos campos AppState); crates/instinct-gui/src/ui/mod.rs (panel de preparación: selector especie, botón entrar, rejilla clicable place/remove, lista colocadas con Quitar); crates/instinct-gui/tests/session/state.rs (campos nuevos en struct literal AppState).
- Decisiones: cero cambios en crates/instinct-core/ (verificado con git diff develop -- crates/instinct-core/ vacío). Receta = {map_name, seed, placements} (decisión 4 de PHASE.md); la GUI guarda la receta, no el mundo. place delega bounds/celda-libre en World::place_creature; remove delega en World::remove_creature.
- Evidencia AC: AC1 tests headless 11/11 OK (colocación válida; objeto/criatura/bounds → Diagnostic sin pánico; quitar deja celda libre; N misma especie; mundo consistente). AC2 receta suficiente (estructura Recipe, F5-08 prueba recreación oficial). AC3 cargo test -p instinct-gui --test prep → 11/11; cargo test --workspace → exit 0; cargo clippy --workspace --all-targets -- -D warnings → exit 0; cargo fmt --all --check → exit 0. AC4 git diff develop -- crates/instinct-core/ vacío; archivos <300 líneas; git status limpio tras commit.