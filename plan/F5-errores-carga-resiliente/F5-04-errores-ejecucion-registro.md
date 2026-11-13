---
id: F5-04
title: Errores de ejecución con archivo real y registro del tick
phase: F5
status: todo
depends_on: []
size: M
---

# F5-04 · Errores de ejecución con archivo real y registro del tick

## Objetivo

Que cada error de ejecución de la sección 2.8 se reporte con el archivo `.ins` real de la criatura y
su línea, que muera solo la criatura que lo cometió y que el error quede en el registro de eventos
del tick para que CLI y ventana lo muestran. Hoy el diagnóstico de ejecución usa el literal
`"interp"` o el nombre de la acción como archivo, y el evento `RuntimeError` no lleva archivo.

## Alcance

**Haz**
- `Species` guarda el origen del archivo compilado (ruta del `.ins` cuando la especie se cargó desde archivo; sin origen cuando se construye en memoria).
- Los errores de ejecución se propagan como `Diagnostic` completo: el archivo real de la criatura sustituye al literal usado hoy por `interp.rs` y al nombre de la acción en `actions.rs`.
- `Event::RuntimeError` gana el campo del archivo de origen; su `Display` usa el formato `file:line: message`.
- `die_by_error` en `sim.rs` usa el diagnóstico real; el evento sigue acompañado de `Died` con causa `error`.
- Una prueba por error de ejecución de 2.8: división por cero, variable leída antes de asignarse, `dx` o `dy` fuera de `{-1, 0, 1}`, mezcla de entero y texto en un operador, texto donde se espera un número (al menos `wait` y `move`).
- Prueba de aislamiento: con dos criaturas en el mismo tick, la que comete el error muere y la otra conserva salud, posición y variables; el error queda entre los eventos del tick.

**No hagas**
- No cambies la semántica de muerte por error (restore desde memento más `Died` causa `error`).
- No cambies los mensajes de error existentes; solo su archivo y línea de origen.
- No reinterpretes qué errores son de ejecución: la lista de 2.8 es fija.
- No toques la CLI: su impresión del evento ya sigue el `Display`.

### Archivos previstos
- `crates/instinct-core/src/species.rs` — modificar — campo de origen del `.ins`.
- `crates/instinct-core/src/resources.rs` — modificar — `load_creatures_folder` pasa el origen al construir la especie.
- `crates/instinct-core/src/interp.rs` — modificar — el `Diagnostic` de ejecución usa el archivo real, que llega por el contexto de evaluación.
- `crates/instinct-core/src/actions.rs` — modificar — los diagnósticos de acción dejan de usar el nombre de la acción como archivo.
- `crates/instinct-core/src/event.rs` — modificar — `RuntimeError` con archivo y `Display` actualizado.
- `crates/instinct-core/src/sim.rs` — modificar — `die_by_error` con el diagnóstico real.
- `crates/instinct-core/tests/sim_tests.rs` — modificar — casos por error de 2.8 y aislamiento.
- `crates/instinct-cli/tests/cli_run.rs` — modificar — asertos que dependan del formato del evento `RuntimeError`, si los hay.

## Criterios de aceptación

- [ ] **AC1** `Event::RuntimeError` incluye el archivo `.ins` de la criatura y su `Display` lo muestra en formato `file:line: message`. Verif: `test_runtime_error_event_carries_file` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC2** Una criatura con una división por cero muere con `RuntimeError` que nombra su `.ins` y la línea de la división. Verif: `test_runtime_error_names_source_file` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC3** Cada error de 2.8 tiene su caso con mensaje y línea correctos: variable sin asignar, dirección inválida, mezcla de tipos y texto donde se espera número. Verif: `cargo test -p instinct-core --test sim_tests --test actions_tests --test interp_tests`.
- [ ] **AC4** Solo muere la criatura que cometió el error: la otra criatura del tick conserva salud, posición y variables, y el error y la muerte quedan en los eventos del tick. Verif: `test_execution_error_kills_only_the_culprit` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC5** La determinismo con semilla fija se mantiene: los cuatro ejemplos del anexo dan la misma historia que antes. Verif: `cargo test -p instinct-core --test annex_examples_tests` y dos corridas idénticas de `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 7 --ticks 30 --place Dwarf@0,1`.
- [ ] **AC6** Suite completa, clippy y fmt en verde. Verif: `cargo test --workspace` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo fmt --all --check`.

## Contexto

**Ya existe**: `sim.die_by_error` emite `RuntimeError` (tick, especie, posición, mensaje, línea) y `Died`; `execute_turn` devuelve el error como `Diagnostic` más línea; `load_creatures_folder` compila con el nombre real del archivo y construye la especie sin guardarlo; los errores de valor (mezcla de tipos, división por cero, overflow) ya nacen en `value.rs` como texto y se convierten a `Diagnostic` en `interp.rs` con archivo falso.

**Contratos que debes respetar**: un error de ejecución mata solo a su criatura y queda en el registro; el intérprete lee y el mundo escribe; `Species` sigue inmutable tras su creación (el origen se fija al construir).

**Ojo**: el camino del archivo hasta el diagnóstico debe ser explícito, no un global: el contexto de evaluación ya recibe la criatura, así que el origen puede viajar desde ahí. Las pruebas que construyen la especie en memoria sin archivo dejan el campo vacío, y el mensaje no debe romperse.

### Leer primero
- `crates/instinct-core/src/interp.rs` — `execute_turn`, `CreatureTurnState` y los diagnósticos con archivo falso.
- `crates/instinct-core/src/sim.rs` — `run_creature_turn` y `die_by_error`.
- `crates/instinct-core/src/species.rs` — estructura inmutable de la especie.
- `crates/instinct-core/src/resources.rs` — `load_creatures_folder`, donde se conoce el archivo.
- `crates/instinct-core/src/actions.rs` — diagnósticos de `wait`, `move`, `attack` y `consume`.
- `crates/instinct-core/src/event.rs` — `RuntimeError` y su `Display`.

## Especificación

| Caso | Disparador | Resultado observable |
|---|---|---|
| División por cero | la criatura evalúa una división por cero | `RuntimeError` con archivo y línea; criatura muerta |
| Variable sin asignar | lectura de nombre no asignado | igual |
| Dirección inválida | desplazamiento con primera componente fuera de rango | igual, mensaje nombra el argumento |
| Mezcla de tipos | suma de entero y texto | igual |
| Texto donde va número | acción `wait` con argumento de texto | igual, mensaje nombra el argumento |

Formato del evento: `file:line: message`, donde `file` es la ruta del `.ins` y `line` la línea de la
criatura donde ocurrió. Especies construidas en memoria sin archivo de origen: campo vacío, mensaje
legible.

## Tests

Nivel: unitario e integración en `sim_tests.rs` (mundo real, sin mocks del cargador salvo para
fijar el origen). Escribe primero los tests que fallan: hoy el error no lleva archivo y las
afirmaciones de archivo y línea real fallan. Determinismo con semilla fija y criaturas sin `random`.

## Verificación

- `cargo test -p instinct-core --test sim_tests` → esperado: pasan los casos nuevos de 2.8 y el de aislamiento.
- `cargo test -p instinct-core --test annex_examples_tests` → esperado: sin regresiones.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → esperado: exit 0.
- `cargo fmt --all --check` → esperado: exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde.

## Si te bloqueas

Si propagar el origen hasta `interp` exigiera un cambio de firma muy amplio, repórtalo antes de
improvisar: la alternativa válida es que `sim.rs` construya el diagnóstico final con el origen de la
especie, siempre que el camino sea explícito y esté probado. Decisiones que no puedes tomar: cambiar
la semántica de muerte, reescribir mensajes, tocar la CLI.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: comandos ejecutados y resultado, AC por AC.

### Notas de implementación

<!-- Rama, commit, comandos ejecutados y resultado, AC por AC, desviaciones. -->
