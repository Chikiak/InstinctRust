---
id: F5-06
title: Render del snapshot en la ventana
status: todo
phase: F5-ventana
depends_on: [F5-01, F5-02]
size: M
file: f5-06-render-snapshot.md
---

# F5-06 · Render del mundo desde el DTO de solo-lectura

## Leer primero

- `crates/instinct-core/src/world/snapshot_view.rs` (DTO `WorldSnapshot` L14-24,
  `World::snapshot()` L72) y `world/snapshot.rs` si hace falta
- `crates/instinct-core/src/world/draw.rs` (el ASCII de referencia: caracteres de
  terreno, objeto y criatura que la ventana debe respetar)
- `plan/F5-ventana/f5-01-spike-gui-crate.md` (módulos `render/` y `ui/` creados allí)

## Tarea

1. Módulo puro `render/`: función `grid_from_snapshot(&WorldSnapshot) -> GridView`
   (o equivalente) que traduce el DTO a una malla de celdas listas para pintar:
   color/etiqueta por terreno (`terrain_char` + `TerrainInfo`), objeto (`object_char`)
   y criatura (inicial de especie + facción vía `LiveCreature`). Sin eframe aquí.
2. Reglas de render testeables sin ventana:
   - Una criatura tapa al objeto y al terreno de su celda; el objeto tapa al terreno.
   - El mapa de símbolos usa los caracteres del DTO (coherente con el ASCII de
     `World::draw`).
   - Ticks y nombre del mapa del snapshot aparecen como metadatos del `GridView`.
3. Capa `ui/` (eframe): pintar la malla con celdas cuadradas y la geometría celda↔píxel
   ya testeada en F5-01; refresco desde `snapshot()` en cada frame de simulación (y al
   colocar en preparación).
4. Conexión mínima de datos: en modo ventana, tras F5-03/F5-04 la app mantiene un
   `WorldSnapshot` vigente; este issue se limita a la función de render + su prueba y
   a pintarlo desde el estado actual de la app.

## Criterios de aceptación

1. Tests headless: snapshot construido en test (mundo mínimo de `examples/` o
   sintético) produce la malla esperada (prioridad criatura > objeto > terreno,
   símbolos y metadatos correctos).
2. La ventana pinta la malla y se refresca al avanzar ticks (verificación manual en
   notas; el headless cubre la lógica).
3. `cargo test --workspace` exit 0; clippy/fmt verdes; núcleo intacto; árbol limpio.

## Verificación

```powershell
cargo test -p instinct-gui
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p instinct-gui   # manual: la rejilla refleja el mundo
```

## Notas de implementación

<!-- Rellena el implementador. -->