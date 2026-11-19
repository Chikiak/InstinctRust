---
id: F5-05
title: Controles de tiempo - play, pausa, paso y velocidades
status: todo
phase: F5-ventana
depends_on: [F5-04]
size: M
file: f5-05-controles-tiempo.md
---

# F5-05 · Controles de tiempo (4.1 paso 3)

## Leer primero

- `crates/instinct-core/src/sim/mod.rs` (`step_tick` L48, `set_rng_seed` L31,
  `tick` L36, `world` L41)
- `plan/F5-ventana/f5-04-preparacion-colocaciones.md` (receta y mundo de preparación)
- Enunciado §4.1 paso 3 (`docs/enunciado.md:386`): pausa, avanzar un tick, al menos
  tres velocidades (ejemplo 2/10/30 ticks por segundo), y no se puede tocar el mundo
  durante la simulación.

## Tarea

1. Estado de simulación (`app/time.rs`), headless:
   - `Speed` con exactamente 2 / 10 / 30 ticks por segundo (decisión 5).
   - `RunState` = `Idle | Running(Speed) | Paused` con transiciones:
     `play`, `pause`, `step_once`, `set_speed`, `back_to_prep` (este último lo
     implementa F5-08).
   - Integración con el reloj de eframe en la capa `ui/`: acumulador de tiempo;
     `update` llama a `step_once` las veces que correspondan según velocidad y delta.
     **Toda** la lógica de acumulación vive en un módulo puro con test
     `advance(elapsed_ms) -> n_ticks` (sin widget).
2. Invariante del enunciado: en `Running`/`Paused` las operaciones de preparación
   (`place`, `remove`) devuelven error "not in preparation" (nunca mutean el mundo).
3. Eventos de cada tick alimentan el registro existente (`LogBuffer` vía el puerto);
   el panel de registro se actualiza con los eventos del tick actual.
4. Capa `ui/`: botones play/pausa, botón de paso, selector de las 3 velocidades y
   contador de tick visible.

## Criterios de aceptación

1. Tests headless: `advance` produce el número correcto de ticks para 2/10/30 tps
   (incl. deltas grandes sin desbordar el bucle); transiciones válidas/ inválidas del
   estado; `place`/`remove` rechazadas fuera de preparación.
2. Un test de humo: con una receta de `examples/` y semilla fija, 10 ticks vía
   `advance` producen los mismos eventos que 10 `step_tick` directos sobre el núcleo
   (los mismos textos de registro).
3. `cargo test --workspace` exit 0; clippy/fmt verdes; núcleo intacto; árbol limpio.

## Verificación

```powershell
cargo test -p instinct-gui
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p instinct-gui   # manual: play/pausa/paso/velocidades
```

## Notas de implementación

<!-- Rellena el implementador. -->