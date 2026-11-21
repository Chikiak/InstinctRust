---
id: F5-09
title: Correcciones de UI reportadas por el usuario (post-entrega F5-01..F5-08)
status: todo
phase: F5-ventana
depends_on: [F5-08]
size: M
file: f5-09-ui-fixes.md
---

# F5-09 · Correcciones de la interfaz reportadas manualmente (post-entrega)

## Leer primero

- `plan/F5-ventana/PHASE.md` (principio rector y decisiones; la GUI es adaptador, la lógica
  vive en `app/`/`render/` y la capa `ui/` solo traduce estado↔widgets↔mensajes)
- `crates/instinct-gui/src/ui/mod.rs` (capa eframe) y `src/render/mod.rs`
  (geometría celda↔píxel + `grid_from_snapshot`)
- `crates/instinct-gui/src/app/mod.rs` (AppState/Msg), `app/session.rs` (catálogos),
  `app/prep.rs` (colocaciones), `app/time.rs` (play/pausa/paso/velocidades)
- `plan/F5-ventana/DEMO_MANUAL.md` (los 4 pasos verificables a mano)

## Origen

Reporte del usuario tras abrir la ventana real (2026-11-20). La entrega F5-01..F5-08 dejó
los gates headless en verde, pero la capa visual (nunca verificada a mano durante la fase)
presenta fallos concretos. Este issue los corrige **sin tocar el núcleo**.

## Síntomas reportados (cada uno es un criterio de aceptación)

1. **Rejilla mal dibujada:** al cargar el mundo, las casillas se ven "movidas de lugar"
   (origen/offset/dimensiones incoherentes con el mapa real).
2. **Colocación ignora la especie elegida:** aunque se seleccione otra especie en el
   desplegable, siempre coloca "Attacker" (existe `examples/creatures/attacker.ins`;
   la selección no surte el estado de la colocación).
3. **Play no muestra movimiento:** al dar play no se ve nada moverse; tras X ticks las
   criaturas desaparecen (refresco por frame del snapshot y representación de muertes).
4. **Estética:** la UI es fea; layout/espaciados/colores mejorables.

## Tarea

1. Diagnosticar cada síntoma sobre el código de `crates/instinct-gui/` (nunca sobre el
   núcleo). Para el síntoma 3, contrastar con la CLI a misma semilla para distinguir
   "historia real del núcleo" de "fallo de refresco/representación".
2. Corregir dentro de `crates/instinct-gui/` (ui/, render/, app/, main.rs):
   - Geometría: celdas alineadas con el mapa; una celda clicada coloca exactamente en
     esa coordenada x,y (ida y vuelta click→celda→mundo verificada).
   - Selección: la especie elegida en el desplegable es la que se coloca; estado de
     selección coherente al entrar en preparación.
   - Refresco: `ui::update` pinta desde un snapshot vigente en cada frame; los ticks se
     reflejan; las muertes se representan (mensaje en registro/glifo), no desaparecen en
     silencio.
   - Estética: jerarquía visual clara (cabecera, rejilla, catálogos, registro),
     espaciados y tamaños coherentes, estados (running/paused) distinguibles.
3. Tests headless que fijen la corrección (patrón `tests/<area>/mod.rs` del crate):
   click→celda→coordenada de mundo; selección de especie surte colocación; refresco por
   frame refleja el snapshot tras `step_once`; la receta sigue determinista.
4. Actualizar `DEMO_MANUAL.md` si cambia la interacción observable.
5. Mantener `cargo run -p instinct-gui -- --smoke` en exit 0.

## Criterios de aceptación

1. Rejilla alineada con el mapa: colocación por clic en (x,y) aparece en esa misma (x,y)
   del mundo (test headless + verificación manual del usuario).
2. Colocar Dwarf, Ent, Hobbit y Uruk (cada uno seleccionado en el desplegable) funciona;
   no aparece colocación de especie no seleccionada (test + manual).
3. En play el movimiento se ve tick a tick; los ticks avanzan; las muertes se representan
   y coinciden con la historia de la CLI a misma semilla (determinismo preservado:
   `--smoke` y tests verdes).
4. Layout legible y coherente (manual; subjetivo pero verificable con los 4 pasos de
   DEMO_MANUAL.md).
5. `cargo test --workspace` exit 0; `cargo clippy --workspace --all-targets -- -D warnings`
   exit 0; `cargo fmt --all --check` exit 0.
6. **Núcleo intacto:** `git diff develop -- crates/instinct-gui/` puede cambiar, pero
   `git diff develop -- crates/instinct-core/` debe estar VACÍO.
7. Ningún `.rs` > ~300 líneas salvo excepción anotada; `git status` limpio tras commit.

## Verificación

```powershell
cargo test -p instinct-gui
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p instinct-gui -- --smoke      # exit 0
cargo run -p instinct-gui                 # manual: repetir DEMO_MANUAL.md 4 pasos
```

## Notas de implementación

<!-- Rellena el implementador: causa raíz por síntoma y fix. -->