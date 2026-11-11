---
id: F4-06
title: Memento del estado de criatura para el turno
phase: F4
status: done
depends_on: [F4-02]
size: S
---

# F4-06 · Memento del estado de criatura para el turno

## Objetivo

Formalizar el patrón Memento/Snapshot (sección 3.4 de `docs/PATRONES_DISENO_ARQUITECTURA.md`) para el estado de criatura al inicio de cada turno: guardar una copia del estado relevante antes de ejecutar, para poder restaurar o persistir cambios atómicamente. `sim.rs` ya clona manualmente la criatura; este issue crea el `CreatureMemento` (o reusa `CreatureSnapshot`) como memento formal, sin cambiar el comportamiento observable. Depende de F4-02 porque captura el `CreatureState`.

## Alcance

**Haz**
- Definir `CreatureMemento` en `crates/instinct-core/src/world.rs` (o reusar/extender `CreatureSnapshot`) que capture el estado mutable de una criatura al inicio del turno: `health`, `age`, `x`, `y`, `pc`, `sleep_turns`/`CreatureState`, `variables` y `id`.
- `sim.run_creature_turn` crea un memento antes de ejecutar el turno de la criatura; si el turno termina con error (Diagnostic de ejecución), se puede restaurar el memento (deshacer) o se documenta qué se conserva. El mundo nunca queda en estado intermedio corrupto.
- `CreatureSnapshot` (ya existente, de solo lectura para percepciones) sigue usándose por las percepciones; el memento es la copia de escritura para el turno.
- `end_of_tick_creatures` aplica los cambios del turno (o los descarta si se restaura) de forma atómica por criatura.
- Mismo comportamiento: misma semilla, misma historia; una acción que falle mata a la criatura con su evento (semántica de F2/F3), no deja al mundo corrupto.
- Pruebas: turno que falla a mitad → la criatura se restaura al memento o muere limpiamente; las variables no se filtran a un turno fallido; misma semilla = misma historia.

**No hagas**
- No cambies la semántica de fin de tick ni el orden de turnos.
- No cambies qué acciones consumen turno.
- No introduzcas `unwrap`/`expect`/`panic!` sobre datos de entrada.
- No añadas deshacer ilimitado: el memento es por turno, no un historial.

### Archivos previstos
- `crates/instinct-core/src/world.rs` — modificar — `CreatureMemento` (o extensión de `CreatureSnapshot`), captura y restauración de estado de turno.
- `crates/instinct-core/src/sim.rs` — modificar — `run_creature_turn` crea memento antes de ejecutar y aplica/restaura atómicamente.
- `crates/instinct-core/src/interp.rs` — verificar que `execute_turn` no muta el mundo directamente (sigue devolviendo `PendingAction` + estado).
- `crates/instinct-core/tests/sim_tests.rs` — modificar — turno fallido no corrompe al mundo.
- `crates/instinct-core/tests/world_tests.rs` — modificar — memento captura y restaura estado.

## Criterios de aceptación

- [ ] **AC1** Existe `CreatureMemento` que captura el estado mutable de una criatura al inicio del turno (salud, edad, posición, pc, sueño, variables, id). Verif: `cargo test --workspace`.
- [ ] **AC2** `sim.run_creature_turn` crea un memento antes de ejecutar; si el turno falla con error de ejecución, el estado de la criatura se restaura o muere limpiamente sin dejar el mundo corrupto. Verif: `test_turn_failure_rolls_back_or_dies_cleanly` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC3** Las variables de una criatura no se filtran parcialmente tras un turno fallido (o la criatura muere con causa registrada). Verif: `test_variables_not_leaked_on_failed_turn` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC4** Determinismo: misma semilla, misma historia (mapa, eventos, muertes) antes y después del issue. Verif: dos corridas `--seed 7`.
- [ ] **AC5** Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Suite completa + clippy + fmt en verde.

## Contexto

**Ya existe**: `CreatureSnapshot<'a>` (solo lectura, para percepciones) en `world.rs`; `sim.run_creature_turn` clona la criatura manualmente antes de ejecutar; `execute_turn` devuelve `Result<TurnResult, (Diagnostic, usize)>` y nunca muta el mundo directamente (el intérprete lee, el mundo escribe). I3/State ya modelado (F4-02).

**Contratos que debes respetar**: el intérprete lee y el mundo escribe; `PendingAction` lo aplica el mundo; una acción que falle mata a la criatura con evento `Died` (causa "error") y `RuntimeError` en el bus (F4-01).

**Ojo**: el memento es por turno y por criatura, no un historial global. Si el turno tiene éxito, los cambios se aplican; si falla, se restaura o la criatura muere. Decide en el issue cuál es la política y documentala.

### Leer primero
- `crates/instinct-core/src/world.rs` — `CreatureSnapshot`, `Creature`, `apply_action_result`.
- `crates/instinct-core/src/sim.rs` — `run_creature_turn`, `step_tick`, clon manual de criatura.
- `crates/instinct-core/src/interp.rs` — `execute_turn`, `TurnResult`, `PendingAction`.
- `crates/instinct-core/tests/sim_tests.rs` — pruebas existentes de simulación.

## Especificación

```
CreatureMemento {
  id: CreatureId,
  health: i64, age: i64, x: usize, y: usize,
  pc: usize, state: CreatureState,  // de F4-02
  variables: HashMap<String, Value>,
}

sim.run_creature_turn(id):
  memento = creature(id).memento()          // captura estado inicial
  result = execute_turn(...)                // intérprete lee, devuelve PendingAction o error
  match result:
    Ok(turn)   → world.apply_action_result(id, turn.pending)   // aplica cambios
    Err(err)   → world.restore(id, memento)  O world.kill(id, err)  // política: restaurar o morir
```

## Tests

Nivel: unitario e integración. TDD: escribe primero el test que falle (un turno fallido deja variables filtradas o el mundo corrupto), luego implementa.

- `world_tests`: `memento()` captura estado; `restore(memento)` devuelve salud, edad, posición, pc, sueño y variables al valor capturado.
- `sim_tests`: criatura hace `attack` con daño invocado que falla → su estado no se filtra parcialmente (o muere con `Died`); el resto del mundo sigue intacto.
- `annex_examples_tests`: `--seed 7` sobre los cuatro ejemplos → misma historia que la versión base.

## Verificación

- `cargo test -p instinct-core --test world_tests` → memento captura/restaura pasa.
- `cargo test -p instinct-core --test sim_tests` → turno fallido sin corrupción pasa.
- `cargo test -p instinct-core --test annex_examples_tests` → no-regresión de semilla 7.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo dan la misma historia que antes del issue.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si `restore` es complejo por `variables: Rc<RefCell<...>>`, para y repórtalo: la alternativa válida es que un turno fallido mate a la criatura (semántica existente) sin necesidad de restaurar todo; el memento entonces solo registra el estado para diagnóstico, no para deshacer. No uses `Rc<RefCell>` nuevo ni `clone` profundo descontrolado.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f4-06-memento-estado-turno (sobre develop). Depende de F4-02.
Commit: pendiente (se hará al cerrar el issue).

Cambios realizados:
- `crates/instinct-core/src/world.rs`: añadido `struct CreatureMemento { id, health, age, x, y, pc, state, variables }` con `Creature::memento()` para capturar estado y `World::restore_creature_from_memento(&memento) -> bool` para restaurar.
- `crates/instinct-core/src/sim.rs`: `run_creature_turn` captura un `CreatureMemento` al inicio del turno; en error de ejecución restaura desde el memento y luego llama `die_by_error` (muerte limpia con eventos `RuntimeError` + `Died` causa "error").

Comandos ejecutados y resultados:
- `cargo test -p instinct-core --test world_tests` → 18 passed (incluye test_creature_memento_captures_state, test_restore_creature_from_memento, test_restore_creature_from_memento_unknown_id).
- `cargo test -p instinct-core --test sim_tests` → 19 passed (incluye test_turn_failure_rolls_back_or_dies_cleanly, test_variables_not_leaked_on_failed_turn).
- `cargo test --workspace` → todo en verde (0 fallos en todos los crates/tests).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings.
- `cargo fmt --all --check` → exit 0, sin diffs.
- Determinismo seed 7: `test_annex_seed7_sequence_unchanged` passed (misma historia en dos corridas).

Evidencia AC por AC:
- AC1 PASS: `CreatureMemento` existe en world.rs con todos los campos (id, health, age, x, y, pc, state, variables). Verificado con test_creature_memento_captures_state.
- AC2 PASS: `run_creature_turn` crea memento antes de ejecutar; en error restaura y muere limpiamente. Verificado con test_turn_failure_rolls_back_or_dies_cleanly.
- AC3 PASS: variables no se filtran tras turno fallido (criatura muere, su mapa de variables desaparece). Verificado con test_variables_not_leaked_on_failed_turn.
- AC4 PASS: determinismo con seed 7 (dos corridas iguales). Verificado con test_annex_seed7_sequence_unchanged.
- AC5 PASS: cero unwrap/expect/panic! nuevos; suite + clippy + fmt en verde.

Desviaciones: ninguna. Política adoptada: en error de ejecución, restaurar desde memento y luego morir limpiamente (semántica de F2/F3 mantenida).