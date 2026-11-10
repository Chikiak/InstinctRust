---
id: F4-03
title: Estrategia de PRNG inyectable en pruebas
phase: F4
status: todo
depends_on: []
size: S
---

# F4-03 · Estrategia de PRNG inyectable en pruebas

## Objetivo

Hacer inyectable el generador pseudoaleatorio mediante el patrón Strategy (sección 8.1 de `docs/PATRONES_DISENO_ARQUITECTURA.md`) para que los tests de simulación no dependan del estado global del mundo. La secuencia con semilla fija no cambia: misma semilla, misma historia.

## Alcance

**Haz**
- Definir en `crates/instinct-core/src/rng.rs` el trait `RngSource { fn next_bounded(&mut self, max: i64) -> i64; fn shuffle<T>(&mut self, list: &mut [T]); }` (añade `next_u64` si lo necesita el orden de turnos).
- `Rng` (SplitMix64 existente) implementa `RngSource`.
- `World` y `Simulation` (`crates/instinct-core/src/world.rs`, `sim.rs`) usan `Box<dyn RngSource>` (o un genérico) en lugar de `RefCell<Rng>` directo, de modo que los tests inyecten un RNG de prueba determinista sin tocar el estado global del mundo.
- `World::set_rng_seed` y `Simulation::set_rng_seed` siguen funcionando creando un `Rng::new(seed)` y envolviéndolo en el trait.
- El orden de turnos (shuffle con el RNG del mundo) y `random` (percepción) usan el `RngSource` del mundo; la secuencia de extracciones con semilla fija es idéntica a antes.
- Pruebas: un RNG de prueba (`TestRng`) que devuelve una secuencia fija; verificar que `shuffle` y `next_bounded` lo usan; la simulación con `TestRng` es determinista; la secuencia con semilla fija del `Rng` real no cambia (prueba de no-regresión de los cuatro ejemplos).

**No hagas**
- No cambies el algoritmo SplitMix64 ni la semilla `u64`.
- No cambies el orden de turnos ni el consumo de `random` por percepción.
- No introduzcas `unwrap`/`expect`/`panic!` sobre datos de entrada.
- No añadas el crate `rand` (prohibido en el núcleo).

### Archivos previstos
- `crates/instinct-core/src/rng.rs` — modificar — trait `RngSource`, `Rng` lo implementa.
- `crates/instinct-core/src/world.rs` — modificar — `rng: Box<dyn RngSource>`, `set_rng_seed`, `next_bounded`/`shuffle` vía trait.
- `crates/instinct-core/src/sim.rs` — modificar — usa el RNG del mundo vía trait (sin cambios de comportamiento).
- `crates/instinct-core/tests/rng_tests.rs` — modificar — `TestRng` y pruebas del trait.
- `crates/instinct-core/tests/sim_tests.rs` — modificar — simulación con `TestRng` inyectado.
- `crates/instinct-core/tests/annex_examples_tests.rs` — modificar — no-regresión de secuencia con semilla fija.

## Criterios de aceptación

- [ ] **AC1** Existe `pub trait RngSource { next_bounded; shuffle }` y `Rng: RngSource`. Verif: `cargo test --workspace`.
- [ ] **AC2** `World` y `Simulation` usan `Box<dyn RngSource>`; `set_rng_seed(seed)` crea `Rng::new(seed)` envuelto. Verif: `test_world_uses_rng_source_trait` | `cargo test -p instinct-core --test rng_tests`.
- [ ] **AC3** Un `TestRng` inyectado en una simulación controla `shuffle` (orden de turnos) y `next_bounded` (percepción `random`); resultado determinista. Verif: `test_simulation_with_injected_rng_is_deterministic` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC4** No-regresión: `Rng::new(7)` produce la misma secuencia de `next_bounded` y `shuffle` que antes del issue (los cuatro ejemplos del anexo dan la misma historia con `--seed 7`). Verif: `test_annex_seed7_sequence_unchanged` | `cargo test -p instinct-core --test annex_examples_tests`.
- [ ] **AC5** Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Suite completa + clippy + fmt en verde.

## Contexto

**Ya existe**: `Rng { state: u64 }` con `new(seed)`, `next_u64`, `next_bounded(max: i64)`, `shuffle<T>` en `rng.rs` (SplitMix64); `World.rng: RefCell<Rng>`; `Simulation.set_rng_seed(seed)`; percepción `random` consume una extracción; orden de turnos hace `shuffle` de los ids vivos.

**Contratos que debes respetar**: misma semilla = misma historia; el núcleo sigue sin dependencias externas; `Rng` sigue siendo el PRNG por defecto.

**Ojo**: `RefCell<Rng>` se reemplaza por `Box<dyn RngSource>`. El interior del mundo sigue siendo de un solo hilo; no añadas `Mutex`. `dyn` tiene un coste mínimo aceptable (decisión 2 de la fase: coste de aprendizaje asumido, F9 mide después).

### Leer primero
- `crates/instinct-core/src/rng.rs` — `Rng`, `next_bounded`, `shuffle`.
- `crates/instinct-core/src/world.rs` — `rng: RefCell<Rng>`, `set_rng_seed`, `next_bounded` usado por percepciones.
- `crates/instinct-core/src/sim.rs` — `step_tick` usa `world.rng` para el shuffle.
- `crates/instinct-core/tests/rng_tests.rs` — pruebas existentes del PRNG.

## Especificación

```
rng.rs
  trait RngSource {
    fn next_bounded(&mut self, max: i64) -> i64;
    fn shuffle<T>(&mut self, list: &mut [T]);
  }
  impl RngSource for Rng { ... }  // SplitMix64, sin cambios de secuencia

world.rs
  World { rng: Box<dyn RngSource>, ... }
  set_rng_seed(seed) -> self.rng = Box::new(Rng::new(seed))

sim.rs
  step_tick → world.rng.shuffle(&mut ids)  (vía trait, mismo comportamiento)
```

## Tests

Nivel: unitario e integración. TDD: escribe primero el test que falle (un mundo sin RNG inyectable no acepta `TestRng`), luego implementa.

- `rng_tests`: `TestRng` implementa `RngSource` con secuencia fija; `next_bounded` y `shuffle` lo usan; `Rng::new(7)` secuencia igual que antes.
- `sim_tests`: simulación con `TestRng` inyectado → orden de turnos predecible y determinista.
- `annex_examples_tests`: `--seed 7` sobre los cuatro ejemplos → misma historia que la versión base (capturada antes del issue).

## Verificación

- `cargo test -p instinct-core --test rng_tests` → `RngSource` y `TestRng` pasan.
- `cargo test -p instinct-core --test sim_tests` → RNG inyectado es determinista.
- `cargo test -p instinct-core --test annex_examples_tests` → no-regresión de semilla 7.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo dan la misma historia que antes del issue.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si `Box<dyn RngSource>` complica `Simulation::new(world: World)` (el mundo ya trae su RNG), para y repórtalo: el RNG viaja dentro del `World`, y `Simulation` no lo toca directamente. No uses `RefCell<Rng>` nuevo ni `rand`.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f4-03-prng-inyectable-strategy (sobre develop).
Evidencia AC por AC: pendiente.
Cambios de firma autorizados: `World.rng` de `RefCell<Rng>` a `Box<dyn RngSource>`.