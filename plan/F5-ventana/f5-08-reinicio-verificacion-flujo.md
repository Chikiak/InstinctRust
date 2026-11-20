---
id: F5-08
title: Reinicio a preparación + verificación headless del flujo 4.1
status: done
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

Branch: `feat/f5-08-reinicio-verificacion-flujo`

AC-by-AC evidence:
- AC1 (PASS): `cargo test -p instinct-gui --test reinicio` → `t_reinicio_01` ok. Desde una simulación a mitad de ticks (3 ticks a 2 tps), `Msg::Restart` deja mundo tick 0, `sim=None`, tiempo `Idle`, `time.tick=0`, misma criatura (Talker) re-colocada en (1,0) y semilla 7 preservada; volver a `TickPlay` + el mismo avance reproduce exactamente el mismo tick (`tick_despues == tick_antes`).
- AC2 (PASS): `cargo test -p instinct-gui --test determinismo` → `t_determinismo_01` ok. 5 ticks con `--seed 7` vía `AppState` producen las mismas líneas de registro que 5× `Simulation::step_tick` directo sobre el mismo mundo (la GUI no perturba el determinismo).
- AC3 (PASS): `cargo run -p instinct-gui -- --smoke` → exit 0. Imprime el registro completo del flujo 4.1 (carga → preparación con las 4 criaturas del baseline → play 2/10/30 tps → pausa → step_once → reinicio → puerta de determinismo de 5 ticks seed 7) y termina en `== --smoke OK: flujo 4.1 completo verificado ==`.
- AC4 (PASS): `cargo test --workspace` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all --check` exit 0; `git diff develop -- crates/instinct-core/` vacío (núcleo intacto: 0 dependencias, 0 `println!`) .
- AC5 (PASS): `plan/F5-ventana/DEMO_MANUAL.md` existe con el comando (`cargo run -p instinct-gui` y `--smoke`) y los 4 pasos verificables (carga; preparación con Dwarf/Ent/Hobbit/Uruk; tiempo play/pausa/paso/velocidades; reinicio), cada uno con "esperado" observable y criterios de éxito/fallo.

Determinismo CLI (verificación de la fase, núcleo intacto): `instinct-cli run --root examples --map 'Plain with rocks' --seed 7 --ticks 30 --place Dwarf@1,0 --place Ent@5,1 --place Hobbit@10,0 --place Uruk@12,3` → dos corridas byte a byte idénticas (exit 0, stderr vacío de 0 bytes). SHA256 de la captura en modo texto Windows (CRLF) = `BF6DE0F5EAB874DF037CE3B8AC9B996B4115F55FAF6D77379531E88F00656D09` (coincide con la línea base de F5-ventana). La captura raw LF da `1401F3F9…172C` (mismo contenido, artefacto de captura ya documentado en la fase R2).

Authorized signature changes: none (sin cambios de API del núcleo; `git diff develop -- crates/instinct-core/` vacío).

Modified files:
- `crates/instinct-gui/src/app/mod.rs`: `Msg::Restart` + `AppState::restart()` (recrea el mundo desde la receta: `enter_prep` + `place` por cada `prep.recipe.placements` + `set_rng_seed(seed)`; nunca `World::clone`; deja `sim=None`, `last_events` vacío, tiempo `Idle`/tick 0; empuja `[reinicio] …` al `LogBuffer`).
- `crates/instinct-gui/src/app/log.rs`: `LogBuffer::push` vuelto `pub` (lo usan `restart`/tests para la entrada de reinicio).
- `crates/instinct-gui/src/main.rs`: `run_smoke()` extendido al flujo 4.1 completo + helpers `run_gui_5_ticks_seed7`, `run_direct_5_ticks_seed7`, `print_log`.
- `crates/instinct-gui/src/ui/mod.rs`: botón "Reiniciar" → `Msg::Restart`.
- `crates/instinct-gui/Cargo.toml`: `[[test]] reinicio` y `[[test]] determinismo`.
- `crates/instinct-gui/tests/reinicio/{mod.rs,common.rs}` (new): t_reinicio_01/02/03.
- `crates/instinct-gui/tests/determinismo/{mod.rs,common.rs}` (new): t_determinismo_01.
- `plan/F5-ventana/DEMO_MANUAL.md` (new), `plan/F5-ventana/f5-08-reinicio-verificacion-flujo.md` (status + notas), `plan/F5-ventana/PHASE.md` (estado + registro).

Deviations:
- La puerta de determinismo compara textos de registro (líneas de eventos), alineado con AC2 ("mismos textos"), no un hash de estado.
- `--smoke` imprime el registro con cabecera `=== registro flujo 4.1 (--smoke) (N entradas) ===` y línea final `== --smoke OK: flujo 4.1 completo verificado ==`.
- Fix round (STEP5): `t_reinicio_03` leía `antes` DESPUÉS del reinicio contra el mismo buffer (`restart` añade una entrada, no reemplaza el buffer), lo que hacía `entries.len() > antes.len()` falso siempre; corregido leyendo `antes` antes de `Msg::Restart`. Además, clippy señaló un `needless_borrow` (`&map` → `map`) en `main.rs` y faltaba `cargo fmt`; ambos corregidos (`cargo fmt --all`).