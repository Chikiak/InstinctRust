---
id: F5-03
title: Pantalla de carga y catálogos
status: todo
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

<!-- Rellena el implementador. -->