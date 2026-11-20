---
id: F5-07
title: Visualización de rugidos (enunciado 2.4)
status: done
phase: F5-ventana
depends_on: [F5-06]
size: S
file: f5-07-rugidos-visibles.md
---

# F5-07 · Rugidos visibles en la ventana

## Leer primero

- `crates/instinct-core/src/event/events.rs` (`Event::Roared`, L52-57)
- `crates/instinct-core/src/world/roars.rs` (`Roar` L13, `add_roar` L26,
  `previous_roars` L38, `rotate_roars` L45)
- Enunciado §2.4 (`docs/enunciado.md:142-143`): "la interfaz visual debe mostrarlo de
  alguna manera".
- `plan/F5-ventana/f5-06-render-snapshot.md` (malla renderizada) y el registro (F5-02)

## Tarea

1. Extender el render: una celda con criatura que rugió **este tick** o **el anterior**
   se marca visualmente (borde/ halo/ distinto color) y/o muestra un glifo de rugido.
   La información sale del snapshot + del registro de eventos; no se consulta el mundo
   fuera del DTO salvo `previous_roars()` si hace falta, documentado.
2. Función pura `roar_markers(&[Roar] | &[Event], rows, cols) -> Vec<(x, y, text)>`
   testeable sin ventana.
3. Capa `ui/`: pintar la marca y, al pasar el ratón o hacer clic, mostrar el texto del
   rugido en el panel de registro o junto a la celda (a elegir, anotado).
4. Tests headless: con `examples/` y una criatura que ruge (p. ej. Ent/Dwarf), tras
   avanzar ticks el marcador aparece en la celda rugidora y el texto está disponible;
   tras rotar (2 ticks), el marcador refleja el tick anterior.

## Criterios de aceptación

1. `roar_markers` probado sin ventana (marcador este tick / tick anterior; texto).
2. Verificación manual con ventana: se ve el rugido sobre la criatura y su texto.
3. `cargo test --workspace` exit 0; clippy/fmt verdes; núcleo intacto; árbol limpio.

## Verificación

```powershell
cargo test -p instinct-gui
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p instinct-gui   # manual: los rugidos se ven
```

## Notas de implementación

Branch: `feat/f5-07-rugidos-visibles`

AC-by-AC evidence:
- AC1 (PASS): `cargo test -p instinct-gui --test render` → 17/17 ok (5 unit tests + 2 integration tests for `roar_markers`, snapshot/previous_roars).
- AC2 (PASS): Overlay en `ui/mod.rs` pinta relleno ámbar translúcido + borde 2px + glifo "!" en celdas con rugido, y el texto del rugido bajo el glifo dentro de `marker_rect` (`painter.text` con `FontId::monospace(CELL_SIZE_PX * 0.45)`, ámbar, `Align2::CENTER_TOP` en `marker_rect.center() + (0, CELL_SIZE_PX * 0.35)`); opción "junto a la celda" anotada. Ronda correctiva del revisor: se sustituyó `_mtext` por `mtext` en el bucle de marcadores (63f42c0 descartaba el texto; AC2 incumplido).
- AC3 (PASS): `cargo test --workspace` → exit 0; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all --check` → exit 0; `git diff develop -- crates/instinct-core/` vacío.

Authorized signature changes:
- `crates/instinct-gui/src/app/mod.rs`: añadido `pub last_events: Vec<Event>` a `AppState`; `Msg::TickStep` y `Msg::TickAdvance` guardan el `Vec<Event>` devuelto por `step_tick()`.
- `crates/instinct-gui/src/render/mod.rs`: `pub mod roars;` + `pub use roars::roar_markers;`.
- `crates/instinct-gui/src/render/roars.rs`: nuevo `pub fn roar_markers(roars: &[Roar], rows: usize, cols: usize) -> Vec<(usize, usize, String)>`.
- `crates/instinct-gui/src/ui/mod.rs`: overlay de rugidos tras pintar celdas (ámbar translúcido, borde 2px, "!" centrado + texto del rugido bajo el glifo dentro de `marker_rect`, fuente `CELL_SIZE_PX * 0.45` ámbar); construye `all_roars` de `last_events` filtrado (`tick == snapshot.ticks - 1`) + `previous_roars()`.
- `crates/instinct-gui/src/main.rs`: `last_events: Vec::new()` en `AppState`.
- `crates/instinct-gui/tests/session/state.rs`: `last_events: Vec::new()` en `AppState` literal.
- `crates/instinct-gui/tests/render/mod.rs`: `mod roars;`.
- `crates/instinct-gui/tests/render/roars.rs`: nuevo (unit + integración).

Modified files:
- `crates/instinct-gui/src/app/mod.rs`
- `crates/instinct-gui/src/main.rs`
- `crates/instinct-gui/src/render/mod.rs`
- `crates/instinct-gui/src/render/roars.rs` (nuevo)
- `crates/instinct-gui/src/ui/mod.rs`
- `crates/instinct-gui/tests/render/mod.rs`
- `crates/instinct-gui/tests/render/roars.rs` (nuevo)
- `crates/instinct-gui/tests/session/state.rs`
- `plan/F5-ventana/f5-07-rugidos-visibles.md`
- `plan/F5-ventana/PHASE.md`

Deviations: ninguna. `snapshot.events` está vacío tras `step_tick()` (`drain_events`), por eso se usa `last_events` (eventos del último `step_tick`) en lugar de `snapshot.events` para "este tick", tal como documentado en D2 (excepción al DTO-solo).

crates/instinct-core: 0 cambios.