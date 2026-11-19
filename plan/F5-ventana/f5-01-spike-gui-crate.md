---
id: F5-01
title: Spike egui/eframe offline + crate instinct-gui en el workspace
status: done
phase: F5-ventana
depends_on: []
size: M
file: f5-01-spike-gui-crate.md
---

# F5-01 · Spike de la biblioteca gráfica y alta del crate GUI

## Leer primero

- `plan/F5-ventana/PHASE.md` (principio rector y decisiones de la fase)
- `Cargo.toml` raíz (workspace `members`, L3-6) y `crates/instinct-cli/Cargo.toml`
- `plan/F5-ventana/INFORME_03_DESACOPLAMIENTO.md` (reglas de límites y decisión de biblioteca)
- `crates/instinct-core/Cargo.toml` (0 dependencias: el núcleo no cambia)

## Tarea

1. **Descargar ahora que hay red** las dependencias de `egui`/`eframe` (versión estable
   actual) con `cargo add`/`cargo fetch` o creando el crate; después, el build debe
   funcionar **sin conexión** (objetivo: `cargo build --offline`/sin red funciona).
   Registra en "Notas de implementación" la versión elegida y el árbol de dependencias
   relevante.
2. Crear `crates/instinct-gui` con su `Cargo.toml`: dependencias `instinct-core`
   (`path = "../instinct-core"`) y las gráficas; añadir `"crates/instinct-gui"` a
   `members` del `Cargo.toml` raíz.
3. Estructura inicial del crate (módulos, no todo en un archivo):
   - `src/lib.rs` — fachada pública: `pub mod app;` y reexports de estado.
   - `src/app/mod.rs` — máquina de estados **sin conocer eframe** (vacía por ahora,
     define los tipos `AppState` y el mensaje `Msg` que issues posteriores llenarán).
   - `src/main.rs` — binario `instinct-gui`: arranca `eframe` con la app; acepta
     `--smoke` (flujo headless, sin ventana; se implementa en F5-08).
4. **Spike visible:** una ventana `eframe` que dibuja una rejilla de celdas cuadradas
   (tamaño derivado de constantes del módulo de render, aún sin datos de mundo) y
   **responde a un clic** mostrando en el título (o en un panel mínimo) la celda
   clicada. El spike queda integrado en el módulo `ui/` (traducción estado→widgets) y
   `render/` (geometría pura, testeable), nunca lógica de simulación en la capa eframe.
5. Tests del crate (sin abrir ventana): `[[test]]` propio en su `Cargo.toml` con el
   patrón `tests/<area>/mod.rs` del repo; primer test verifica la geometría pura
   celda↔píxel del módulo `render` (ida y vuelta).

## Criterios de aceptación

1. `cargo build --workspace` compila con el crate nuevo y `cargo test --workspace`
   pasa (exit 0) incluyendo los tests de `instinct-gui`.
2. El build funciona **sin red** (verificado: `cargo build --offline` o con red
   desconectada; evidencia en las notas).
3. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0;
   `cargo fmt --all --check` → exit 0.
4. El núcleo sigue intacto: `crates/instinct-core/Cargo.toml` con 0 dependencias y
   `git status` sin cambios en `crates/instinct-core/`.
5. La ventana spike dibuja rejilla y reporta el clic; la geometría celda↔píxel tiene
   test que falla antes y pasa después.
6. Ningún archivo `.rs` nuevo pasa ~300 líneas salvo excepción anotada.
7. `git status --short` limpio tras commit.

## Verificación

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo build --offline -p instinct-gui   # sin red
cargo run -p instinct-gui               # ver la ventana spike (manual, cerrar)
```

## Notas de implementación

- Versión elegida: `eframe = "0.33.3"` + `egui = "0.33.3"` (una sola copia de `egui`; `cargo add eframe` resolvió 0.33.3 porque la 0.37.0 requiere rustc 1.95 y el toolchain es 1.90.0).
- Árbol de dependencias (`cargo tree -p instinct-gui -e normal --depth 1`): `instinct-gui` → `eframe v0.33.3`, `egui v0.33.3`, `instinct-core v0.0.0` (path).
- Evidencia offline: `cargo build --offline -p instinct-gui` → exit 0 (crates en caché tras `cargo fetch` con red).
- `--smoke`: imprime a stderr y termina con exit 0 (flujo headless real en F5-08). Flag desconocido: usage a stderr + exit 2 (estilo `instinct-cli`).
- Tamaño de ventana: `GRID_WIDTH_PX + 2*MARGIN_PX` × `GRID_HEIGHT_PX + 2*MARGIN_PX` = 512×400 px (rejas de 16×12 celdas de 32 px con margen 8 px).
- Módulos nuevos: `src/render/mod.rs` (~70 L), `src/app/mod.rs` (~35 L), `src/ui/mod.rs` (~100 L), `src/main.rs` (~45 L), `src/lib.rs` (~15 L), `tests/render/geometry.rs` (~100 L), `tests/render/mod.rs` (~10 L). Ninguno supera las ~300 L.
- `crates/instinct-core`: sin cambios (0 dependencias, 0 E/S).