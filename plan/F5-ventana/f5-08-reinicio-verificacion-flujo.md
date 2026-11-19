---
id: F5-08
title: Reinicio a preparación + verificación headless del flujo 4.1
status: todo
phase: F5-ventana
depends_on: [F5-04, F5-05]
size: M
file: f5-08-reinicio-verificacion-flujo.md
---

# F5-08 · Reinicio y puerta de salida headless del flujo completo

## Leer primero

- `crates/instinct-core/src/world/world.rs` (`from_map` L71) y `world/clone_debug.rs`
  L7-28 (`World::clone` resetea RNG a 0 y quita el puerto → **no usar** para reiniciar)
- `plan/F5-ventana/f5-04-preparacion-colocaciones.md` (receta) y `f5-05-controles-tiempo.md`
- Enunciado §4.1 paso 4 (`docs/enunciado.md:388`): "Un botón devuelve a la fase de
  preparación".
- `plan/F5-ventana/PHASE.md` (verificación de salida de la fase)

## Tarea

1. `restart() -> PrepState` en la máquina de estados: **recrear** el mundo desde la
   receta (mismo mapa, mismas colocaciones, misma semilla vía `set_rng_seed`), nunca
   `World::clone`. Queda en preparación con el registro indicando el reinicio.
2. **Modo `--smoke`** del binario `instinct-gui` (sin ventana): ejecuta el flujo 4.1
   completo sobre la máquina de estados y sale con 0:
   carga `examples/` → entra en preparación con el mapa por defecto → coloca las
   criaturas del anexo (Dwarf, Ent, Hobbit, Uruk) en las coordenadas del baseline de
   determinismo → play con las 3 velocidades → pausa → `step_once` → reinicio →
   verifica que la receta recreada produce el mismo estado inicial. Cualquier fallo
   imprime la causa al registro y sale con 1.
3. **Puerta de determinismo de la GUI:** en `--smoke`, tras el reinicio, correr
   5 ticks con `--seed 7` debe dar exactamente los mismos eventos (mismos textos de
   registro) que `Simulation::step_tick` ×5 con semilla 7 sobre la misma colocación.
   Esto se prueba como test de integración (sin ventana), comparando las entradas del
   registro contra las esperadas.
4. Botón de reinicio en la capa `ui/` y mensaje claro en el registro.
5. `plan/F5-ventana/DEMO_MANUAL.md`: guía corta (comando, pasos esperados) para que el
   usuario verifique a mano la demostración visual del flujo 4.1 completo.

## Criterios de aceptación

1. Test: reinicio desde una simulación a mitad de ticks devuelve un mundo con tick 0,
   mismas criaturas colocadas y semilla preservada (las próximas corridas reproducen
   la misma historia).
2. Test: eventos de 5 ticks con `--seed 7` vía la GUI == eventos directos del núcleo
   (mismos textos) — la GUI no perturba el determinismo.
3. `cargo run -p instinct-gui -- --smoke` → exit 0 con registro del flujo completo.
4. `cargo test --workspace` exit 0; clippy/fmt verdes; núcleo intacto; árbol limpio.
5. `DEMO_MANUAL.md` existe y describe los 4 pasos verificables a mano.

## Verificación

```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
cargo run -p instinct-gui -- --smoke     # debe terminar en exit 0
# Determinismo (línea base de la fase): el comando CLI debe seguir dando
# SHA256 BF6DE0F5EAB874DF037CE3B8AC9B996B4115F55FAF6D77379531E88F00656D09 (2 corridas)
instinct-cli run --root examples --map 'Plain with rocks' --seed 7 --ticks 30 --place Dwarf@1,0 --place Ent@5,1 --place Hobbit@10,0 --place Uruk@12,3
cargo run -p instinct-gui                # manual: DEMO_MANUAL.md
```

## Notas de implementación

<!-- Rellena el implementador. -->