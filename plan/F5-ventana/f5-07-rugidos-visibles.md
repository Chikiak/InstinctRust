---
id: F5-07
title: Visualización de rugidos (enunciado 2.4)
status: todo
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

<!-- Rellena el implementador. -->