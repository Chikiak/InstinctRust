---
id: F0-07
title: Turnos y ticks con sus leyes
phase: F0
status: todo
depends_on: [F0-06]
size: M
---

# F0-07 · Turnos y ticks con sus leyes

## Objetivo
Que una `Simulation` avance un tick completo: cada criatura viva ejecuta su turno con las reglas del enunciado y el mundo aplica las leyes de fin de tick. Aquí se cumple "el intérprete lee y el mundo escribe" y es el issue de mayor riesgo de la fase.

## Alcance
**Haz**
- Estado de turno de cada criatura: punto de ejecución, turnos de sueño y variables.
- Intérprete de un turno sobre el `Program` compilado.
- `Simulation` con orden de turnos sorteado, aplicación de la acción pendiente, errores de ejecución y fin de tick.
- Eventos `Died` y `RuntimeError`.

**No hagas**
- Más acciones ni percepciones (F1 y F2).
- Comandos de terminal nuevos (F0-08).
- Reproducción ni nacimientos.

### Archivos previstos
- `crates/instinct-core/src/interp.rs` — nuevo — ejecución de un turno.
- `crates/instinct-core/src/sim.rs` — nuevo — `Simulation` y fin de tick.
- `crates/instinct-core/src/world.rs` — modificar — estado de turno por criatura, regeneración y retirada.
- `crates/instinct-core/src/event.rs` — modificar — `Died` y `RuntimeError`.

## Criterios de aceptación
- [ ] **AC1** Un turno ejecuta etiquetas, `goto`, `if`, asignaciones y `say` sin consumir turno y se detiene tras la primera otra acción; el turno siguiente continúa en la línea siguiente a esa acción. Verif: `test_turn_resumes_after_action`
- [ ] **AC2** Un bucle sin acciones ni final (una etiqueta y un `goto` a ella) termina cada turno al llegar a 100 líneas ejecutadas contando etiquetas, deja viva a la criatura y sigue ejecutándose en los turnos siguientes. Verif: `test_line_limit_ends_turn`
- [ ] **AC3** Al llegar al final del archivo el turno termina sin acción y el siguiente empieza en `start:`. Verif: `test_end_of_program_restarts_at_start`
- [ ] **AC4** `wait` de 3 deja dos turnos siguientes sin ejecutar mientras la criatura sigue perdiendo vida y envejeciendo. Verif: `test_wait_skips_turns_but_creature_ages`
- [ ] **AC5** Al terminar el tick toda criatura pierde 1 de vida y suma 1 de edad; muere con vida 0 (vida inicial 10: al final del tick 10) o con edad igual a la esperanza (esperanza 5: al final del tick 5), con evento `Died` y causa `vida` o `edad`, y desaparece del mapa. Verif: `test_death_by_health_and_by_age`
- [ ] **AC6** El terreno recupera su regeneración sin pasar del máximo y el objeto con reserva 0 desaparece al final del tick (se prepara la reserva con un método de test). Verif: `test_terrain_regenerates_and_empty_objects_vanish`
- [ ] **AC7** Un error de ejecución (división por cero, variable sin asignar, `move` con dirección inválida) mata a esa criatura en el acto con `RuntimeError` que lleva la línea fuente y el mensaje, la quita del mapa y no afecta a las demás. Verif: `test_runtime_error_kills_creature_with_line`
- [ ] **AC8** Con la misma semilla y las mismas colocaciones, 50 ticks producen exactamente los mismos eventos y el mismo mapa; con semillas distintas cambia el orden de turnos (dos criaturas que disputan una casilla terminan distinto en alguna semilla). Verif: `test_same_seed_same_history`, `test_seed_changes_turn_order`
- [ ] **AC9** Suite completa, clippy y fmt en verde. Verif: los comandos de Verificación

## Contexto
**Ya existe**: `Program` y compilación en `crates/instinct-core/src/compile.rs` y `program.rs` (F0-06); mundo con criaturas, acciones, eventos, ticks y `Rng` en `world.rs` (F0-04); percepciones, variables y contexto de evaluación en `perception.rs` (F0-05).
**Contratos que debes respetar**: "Turno de una criatura", "Fin de tick" y "Eventos" en `../PHASE.md#contratos-compartidos`; A3, A4 y A5 en `docs/ambiguedades.md`.
**Ojo**: evaluar un turno necesita leer el mundo y escribir el estado de la criatura a la vez: saca el estado de turno de la criatura (o sepáralo del mundo) mientras se ejecuta y devuélvelo al terminar, y aplica la acción pendiente cuando el intérprete ya soltó la lectura. Las criaturas que mueren en medio del tick no juegan su turno si aún no les tocaba. Las variables que vimos en F0-05 viven ahora en el estado de turno.
**Rust que se practica**: el borrow checker en la práctica (`std::mem::take`, separar lectura y escritura), iterar sobre una copia de identificadores mientras se modifica, `Option`, un bucle de simulación determinista.

### Leer primero
- `crates/instinct-core/src/program.rs` — líneas del programa y números de línea fuente.
- `crates/instinct-core/src/perception.rs` — contexto de evaluación que usará el intérprete.
- `crates/instinct-core/src/actions.rs` — `Action`, consumo de turno y sueño devuelto.
- `crates/instinct-core/src/world.rs` — almacén de criaturas, ticks, eventos.
- `docs/ambiguedades.md` — A3 a A5.

## Especificación
- El intérprete devuelve como máximo una acción pendiente con sus argumentos ya evaluados; el mundo la aplica. `say` se aplica en el acto sin terminar el turno.
- Un error de acción (de F0-04) o de evaluación se convierte en `RuntimeError` con la línea de la línea ejecutada.
- Orden del tick: sortear el orden de las criaturas vivas al comienzo, ejecutar turnos, aplicar fin de tick, subir el contador.
- La causa de muerte por error es `error`.
- Los tests compilan programas desde texto en memoria con el compilador de F0-06 y colocan criaturas en un mundo pequeño.

## Tests
Unitarios y de escenario con `Simulation`; sin mocks del mundo ni del compilador. Escribe primero AC2 y AC8: los que más suelen romperse.

## Verificación
- `cargo test -p instinct-core sim` y `cargo test -p instinct-core interp` → esperado: AC1 a AC8 en verde.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` → esperado: exit 0.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si la separación lectura/escritura te empuja a `Rc<RefCell>` por todo el mundo o a que el intérprete reciba el mundo mutable, para y repórtalo con el error del compilador: no es una decisión tuya. Tampoco ajustes las reglas de turno: salen de la tabla de la fase.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Añade a `docs/ambiguedades.md` cualquier decisión nueva.
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
