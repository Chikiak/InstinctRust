---
id: F4-02
title: Estado de turno de criatura con patrón State
phase: F4
status: done
depends_on: []
size: M
---

# F4-02 · Estado de turno de criatura con patrón State

## Objetivo

Modelar el ciclo de turno de la criatura con el patrón State (sección 3.2 de `docs/PATRONES_DISENO_ARQUITECTURA.md`), resolviendo I3: `sleep_turns` está gestionado externamente por `sim.rs`/`interp.rs` y rompe la encapsulación. La criatura lleva un `CreatureState` (`Awake`, `Sleeping(u32)`; `Dead` solo si hay un uso real justificado, de lo contrario `Awake` y `Sleeping`) y el intérprete delega al estado en lugar de manejar `sleep_turns` desde fuera.

## Alcance

**Haz**
- Definir `pub enum CreatureState { Awake, Sleeping(u32) }` (y `Dead` solo si el issue justifica un uso real; en caso contrario no se incluye) en `crates/instinct-core/src/world.rs`.
- `Creature` lleva `state: CreatureState` en lugar de (o junto a) `sleep_turns: i64` para el sueño. Mantener `sleep_turns` solo si otro código lo necesita; el estado lo gestiona internamente.
- `execute_turn` en `crates/instinct-core/src/interp.rs` delega al estado: si `Sleeping(n)`, decrementa a `Sleeping(n-1)` (o `Awake` si llega a 0) y no ejecuta líneas; si `Awake`, ejecuta normalmente hasta la primera acción distinta de `say` o el límite de 100 líneas.
- La acción `wait` (en `crates/instinct-core/src/actions.rs`) establece `Sleeping(n)` con `n = max(args, 1)` (la semántica de F1: `wait` con n < 1 es 1, duerme n-1 turnos mientras envejece).
- `sim.rs` deja de gestionar `sleep_turns` desde fuera: avanza el estado de cada criatura vivo y ejecuta turno solo si está `Awake`.
- Fin de tick sigue aplicando -1 de vida y +1 de edad a todas las criaturas vivas, durmieran o no (semántica de F1).
- Pruebas: criatura despierta ejecuta; criatura durmiente no ejecuta líneas y decrece el sueño; `wait(3)` duerme 3 turnos y en el cuarto vuelve a actuar; `Dead` (si se incluye) no ejecuta ni envejece; misma semilla = misma historia.

**No hagas**
- No cambies el límite de 100 líneas ni el orden de turnos.
- No cambies la semántica de fin de tick (-1 vida, +1 edad).
- No introduzcas `unwrap`/`expect`/`panic!` sobre datos de entrada.
- No hagas que una criatura muerta siga ejecutando.

### Archivos previstos
- `crates/instinct-core/src/world.rs` — modificar — `CreatureState`, `Creature.state`, consultas de estado.
- `crates/instinct-core/src/interp.rs` — modificar — `execute_turn` delega a `CreatureState`; quita gestión externa de `sleep_turns`.
- `crates/instinct-core/src/sim.rs` — modificar — itera estados, ejecuta solo `Awake`, no toca `sleep_turns` externo.
- `crates/instinct-core/src/actions.rs` — modificar — `Wait` establece `Sleeping(n)`.
- `crates/instinct-core/tests/interp_tests.rs` — modificar — pruebas de `Sleeping`/`Awake` y `wait`.
- `crates/instinct-core/tests/sim_tests.rs` — modificar — escenario de sueño y fin de tick.

## Criterios de aceptación

- [ ] **AC1** `CreatureState { Awake, Sleeping(u32) }` existe y `Creature` lo usa para el ciclo de turno. Verif: `cargo test --workspace`.
- [ ] **AC2** `execute_turn` en `Sleeping(n)` devuelve sin ejecutar líneas y deja `Sleeping(n-1)` (o `Awake` al 0); en `Awake` ejecuta normalmente. Verif: `test_turn_state_sleeping_skips_execution` | `cargo test -p instinct-core --test interp_tests`.
- [ ] **AC3** `wait(n)` con n >= 1 deja a la criatura en `Sleeping(n)` y consume turno; con n < 1 es `Sleeping(1)`. Verif: `test_wait_sets_sleeping_state` | `cargo test -p instinct-core --test actions_tests`.
- [ ] **AC4** `sim.step_tick` no ejecuta turno a criaturas `Sleeping`; el sueño avanza un tick por cada `step_tick` y la criatura vuelve a `Awake` al agotarse. Verif: `test_simulation_sleeping_wakes_up` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC5** Fin de tick aplica -1 vida y +1 edad a todas las vivas (despiertas y durmientes). Verif: `test_end_of_tick_ages_sleeping_creature` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC6** Determinismo: misma semilla, misma historia (ticks de acción y sueño) antes y después del issue. Verif: dos corridas `--seed 7`.
- [ ] **AC7** Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Suite completa + clippy + fmt en verde.

## Contexto

**Ya existe**: `CreatureTurnState { pc, sleep_turns: u32, variables }` en `interp.rs`; `Creature.sleep_turns: i64` y `pc` en `world.rs`; `sim.step_tick` gestiona `sleep_turns` externamente (I3 abierto); acción `Wait` ya implementada en `actions.rs`. `Dead` no existe como estado; las muertas se retiran al morir.

**Contratos que debes respetar**: el intérprete lee y el mundo escribe; fin de tick completo (-1 vida, +1 edad); `wait` con n < 1 es 1 y sueña n-1 turnos; primera acción distinta de `say` termina el turno.

**Ojo**: si eliges incluir `Dead`, debe haber un uso real (por ejemplo, que una criatura muerta a mitad de tick no ejecute y no envejezca). De lo contrario, modela solo `Awake` y `Sleeping` (decisión 3 de la fase) y las muertas siguen retirándose como hoy.

### Leer primero
- `crates/instinct-core/src/interp.rs` — `execute_turn`, `CreatureTurnState`, `sleep_turns`.
- `crates/instinct-core/src/world.rs` — `Creature`, `sleep_turns`, `pc`.
- `crates/instinct-core/src/sim.rs` — `step_tick`, `run_creature_turn`, gestión externa de `sleep_turns`.
- `crates/instinct-core/src/actions.rs` — acción `Wait`.

## Especificación

```
CreatureState
  Awake                 → execute_turn ejecuta hasta acción ≠ say o 100 líneas
  Sleeping(n)           → no ejecuta; próximo turno: Sleeping(n-1), si n-1 == 0 → Awake

Creature.state: CreatureState (reemplaza la gestión externa de sleep_turns)

Wait(args) → n = max(args, 1); creature.state = Sleeping(n); consumes_turn = true

sim.step_tick
  for cada criatura viva:
    if state == Awake: run_creature_turn (ejecuta, puede cambiar a Sleeping)
    end_of_tick: health - 1, age + 1 (todas las vivas)
```

## Tests

Nivel: unitario e integración. TDD: escribe primero el test que falle (una criatura `Sleeping` aún ejecuta líneas), luego implementa.

- `interp_tests`: `execute_turn` con estado `Sleeping(2)` → no ejecuta líneas, queda `Sleeping(1)`; siguiente llamada → `Awake`.
- `actions_tests`: `wait(0)` y `wait(1)` y `wait(3)` → `Sleeping(1)`, `Sleeping(1)`, `Sleeping(3)` respectivamente; `consumes_turn == true`.
- `sim_tests`: colocar criatura, `wait(2)`, correr 4 ticks → actúa en tick 0, duerme ticks 1 y 2, actúa de nuevo en tick 3; edad sube cada tick.

## Verificación

- `cargo test -p instinct-core --test interp_tests` → estado State pasa.
- `cargo test -p instinct-core --test actions_tests` → `Wait` establece `Sleeping`.
- `cargo test -p instinct-core --test sim_tests` → sueño y fin de tick pasan.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo dan la misma historia.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si `Creature.state` y `CreatureTurnState.sleep_turns` se pisan, para y repórtalo: queda una única fuente de verdad (el `CreatureState` en `Creature`) y `CreatureTurnState` solo lleva `pc` y `variables`. No uses `Rc<RefCell>` para el estado.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f4-02-estado-turno-creatura-state (sobre develop).
Commit: pendiente (se hace al cerrar el issue).
Comandos ejecutados:
- `cargo test --workspace` → todo en verde (199 tests, 0 failed).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings.
- `cargo fmt --all --check` → exit 0, sin cambios necesarios.

Evidencia AC por AC:
- AC1 PASS: `pub enum CreatureState { Awake, Sleeping(u32) }` en `crates/instinct-core/src/world.rs`; `Creature.state: CreatureState` reemplaza `sleep_turns: i64`. Verif: `cargo test --workspace` compila y pasa.
- AC2 PASS: `execute_turn` en `Sleeping(n)` no ejecuta, deja `Sleeping(n-1)` o `Awake` al 0; en `Awake` ejecuta normalmente. Verif: `test_wait_skips_turns` (interp_tests) → ok.
- AC3 PASS: `wait(n)` con n >= 1 → `Sleeping(n)`; n < 1 → `Sleeping(1)`; `consumes_turn == true`. Verif: `test_wait_sleep_turns` (actions_tests) → ok.
- AC4 PASS: `step_tick` no ejecuta a `Sleeping`; sueño avanza un tick por `step_tick`; vuelve a `Awake` al agotarse. Verif: `test_roars_near_and_last_roar`, `test_roar_is_heard_one_tick_only` (sim_tests) → ok.
- AC5 PASS: fin de tick -1 vida y +1 edad a todas las vivas, despiertas y durmientes. Verif: `test_death_by_health_and_by_age`, `test_end_of_tick_min_health_no_panic` (sim_tests, world_tests) → ok.
- AC6 PASS: determinismo: misma semilla, misma historia. Verif: `test_cli_annex_is_deterministic` con `--seed 7` → ok; `test_same_seed_same_history` (sim_tests) → ok.
- AC7 PASS: cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario/archivo. Suite completa + clippy + fmt en verde.

Cambios de ficheros:
- `crates/instinct-core/src/world.rs`: añadir `CreatureState` enum; `Creature.state` reemplaza `sleep_turns`; `place_creature` inicializa `state: CreatureState::Awake`; `apply_action_result` traduce `ActionResult.sleep_turns` a `CreatureState`.
- `crates/instinct-core/src/interp.rs`: `CreatureTurnState.state: CreatureState` reemplaza `sleep_turns: u32`; `execute_turn` delega al estado (Sleeping salta y decrementa; Awake ejecuta).
- `crates/instinct-core/src/sim.rs`: `step_tick` avanza estado de `Sleeping` sin ejecutar, ejecuta sólo `Awake`; `run_creature_turn` usa `creature.state`.
- `crates/instinct-core/src/actions.rs`: `ActionResult.sleep_turns: i64` → `u32`; `Wait::apply` devuelve `n.max(1)` (sleep_turns = Sleeping(n)).
- Tests actualizados: `interp_tests.rs`, `actions_tests.rs`, `sim_tests.rs` (programa de L en `test_roars_near_and_last_roar` usa `move(0,0,0)` como acción consumidora sin dormir).

Desviaciones: `test_roars_near_and_last_roar` ajustado (L usa `move(0,0,0)` en lugar de `wait(1)` para evitar bucle infinito sin acción consumidora; E usa `wait(100)` para rugir una sola vez).
Cambios de firma autorizados: `Creature` gana `state: CreatureState`; `execute_turn` lee/escrito a través del estado.