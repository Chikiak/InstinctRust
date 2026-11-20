---
id: F5-03
title: Pantalla de carga y catálogos
status: done
phase: F5-ventana
depends_on: [F5-01, F5-02]
size: M
file: f5-03-carga-catalogos.md
---

# F5-03 · Carga de recursos y catálogos en la app (4.1 paso 1)

## Leer primero

- `crates/instinct-core/src/resources/mod.rs` (`LoadedResources`, `load_resources`)
- `crates/instinct-cli/src/run.rs` L14-46 (orden de carga y diagnósticos a stderr)
- `plan/F5-ventana/INFORME_02_FLUJO_GUI.md` §3 (paso 1: CUMPLE en CLI)
- `plan/F5-ventana/f5-02-adaptador-eventos-registro.md` (el registro recién creado)

## Tarea

1. Estado de app (`app/`): tipo `LoadedSession` que guarda el resultado de
   `instinct_core::resources::load_resources(root)` (root por defecto `examples`,
   configurable por argumento `--root`, igual que la CLI) y el `LogBuffer`.
2. Reglas headless (testeadas sin ventana):
   - Los recursos cargados quedan disponibles aunque haya diagnósticos (nunca abortar):
     mapas y especies se listan aunque algún archivo falle.
   - Cada diagnóstico entra al registro con marca de carga y su línea.
   - Orden de listado: alfabético por nombre (coherente con el orden de carga del
     núcleo, `resources/folder.rs`).
3. Mensajes/estados mínimos para la UI: mapa seleccionado (inicial: el primero),
   especie seleccionada (inicial: la primera), y el estado "sin mapas"/"sin especies"
   con su mensaje en el registro (la app no puede entrar en preparación vacía).
4. Capa `ui/` (eframe): un panel de estado que muestre los catálogos (nombres de mapas
   y especies) y el registro de carga; **sin lógica** más allá de traducir estado a
   widgets y clics a mensajes.
5. Tests de integración sobre el estado headless con fixtures reales de `examples/`
   (vía ruta relativa desde `CARGO_MANIFEST_DIR`, patrón de `cli/tests/cli_annex/
   helpers.rs`): carga correcta lista los mapas y especies esperados; una carpeta con
   un recurso inválido deja el resto disponible y añade el diagnóstico al registro.

## Criterios de aceptación

1. Cargar `examples/` headless produce catálogos no vacíos (mapas: "Plain with rocks";
   especies incluyen Dwarf, Ent, Hobbit, Uruk) y 0 diagnósticos nuevos.
2. Un mapa con carácter desconocido (fixture) se descarta con diagnóstico y línea;
   el resto de mapas sigue disponible; el registro lo muestra.
3. Sin mapas o sin especies, la app no entra en preparación y lo dice en el registro.
4. `cargo test -p instinct-gui` y `cargo test --workspace` exit 0; clippy/fmt verdes.
5. Cero cambios en `crates/instinct-core/`; `git status` limpio tras commit.

## Verificación

```powershell
cargo test -p instinct-gui
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

## Notas de implementación

Branch: `feat/f5-03-carga-catalogos` (base `develop`).

AC-by-AC evidence:
- AC1 (PASS): `cargo test -p instinct-gui --test session` → `catalogs::t1_examples_loads_catalogs_without_diagnostics` ok: `examples/` loads, maps contains "Plain with rocks", species contains Dwarf/Ent/Hobbit/Uruk, 0 load diagnostics.
- AC2 (PASS): `catalogs::t2_unknown_character_produces_diagnostic_and_line` ok: fixture map with unknown char produces diagnostic naming the char with line `:3:`; other maps available; log shows it.
- AC3 (PASS): `catalogs::t3_no_maps_cannot_prepare` / `t4_no_species_cannot_prepare` ok: `can_prepare()` false and log reports "no maps" / "no species".
- AC4 (PASS): `cargo test -p instinct-gui` → exit 0; `cargo test --workspace` → exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all --check` → exit 0.
- AC5 (PASS): `git status --short` shows zero changes under `crates/instinct-core/`.

Authorized signature changes: `AppState` gained `pub session: Option<LoadedSession>`; `Msg` gained `MapSelected(usize)` / `SpeciesSelected(usize)`; `lib.rs` re-exports `app::session::LoadedSession`; `main.rs` parses `--root <dir>` and builds `SpikeApp` with `session: Some(LoadedSession::new(&root))`.

Modified files:
- `crates/instinct-gui/src/app/mod.rs` (session field + Msg variants + update arms)
- `crates/instinct-gui/src/app/session.rs` (new, LoadedSession)
- `crates/instinct-gui/src/lib.rs` (re-export LoadedSession)
- `crates/instinct-gui/src/ui/mod.rs` (side panel: mapas/especies/registro/can_prepare)
- `crates/instinct-gui/src/main.rs` (--root/--smoke args, session init)
- `crates/instinct-gui/Cargo.toml` (add [[test]] session)
- `crates/instinct-gui/tests/session/{mod,common,catalogs,state}.rs` (new tests)
- `plan/F5-ventana/f5-03-carga-catalogos.md` (status + notas)
- `plan/F5-ventana/PHASE.md` (issues table + registro de ejecución)

Deviations: none to the nucleus (`crates/instinct-core` untouched). Note: per core behavior in `resources/map.rs`, a map with an unknown character is *kept* (row retained) with a diagnostic, not discarded; T2 asserts the diagnostic + line + other maps available, matching actual core behavior.
