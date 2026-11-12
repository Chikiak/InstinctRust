# Informe nocturno — F4 · Arquitectura y patrones de diseño

**Fecha:** sesión nocturna 2026-11-10/11 · **Rama base:** `develop` (desde `4dbdbd9`)
**Runbook:** `plan/F4-arquitectura-patrones/RUNBOOK_F4.md` (manda sobre la ficha para lo específico)
**Resultado:** 8/8 issues despachados y mergeados. 7 implementados + 1 descartado documentado (F4-07).

## Resumen por issue

| ID | Patrón | Veredicto | Commit(s) |
|----|--------|-----------|-----------|
| F4-01 | Observer/bus + puertos y adaptadores (I6, 1.2, 4.3) | verde, AC1-AC7 PASS | `09f2441` + merge `2e1e335` |
| F4-02 | State del turno de criatura (I3, 3.2) | verde, AC1-AC7 PASS | `c70617e` + merge `fe0d497` |
| F4-03 | Strategy PRNG inyectable (8.1) | verde, AC1-AC5 PASS | `a265116`+`b739f5b` + merge `3e948b7` |
| F4-04 | ECS ligero, almacén indexado por `CreatureId` (W5, 4.1) | verde, AC1-AC6 PASS | `f24f252` + merge `28567f2` |
| F4-05 | Flyweight `Rc<Species>` compartido (4.2) | verde, AC1-AC5 PASS | `acd16ac`+`21b0189` + merge `31c0c48` |
| F4-06 | Memento del estado de criatura (3.4) | verde, AC1-AC5 PASS | `b27458d` + merge `b8b21f3` |
| F4-07 | Visitor AST (2.2) | **descarte correcto**: sin consumidor real (decisión 4 de la fase + MASTER dec. 40); no se implementa, queda como candidato. AC1/AC3/AC4/AC5 PASS, AC2 N/A | `3048748` + merge `66f860a` |
| F4-08 | Capas y límites (1.1, W2) | verde, AC1-AC6 PASS | `d712f81`+`0e45293` + merge `5d8ea3f` |

Orden seguido (runbook): raíces F4-01 → F4-02 → F4-03 (F4-03 con 1 reanudación por fallo de infraestructura, misma sesión, primer intento limpio) → F4-04 → F4-05 → F4-06 → F4-07 → F4-08.

## Verificación final (ejecutada por el orquestador sobre `develop` mergeado)

- `cargo test --workspace` → **0 fallos** (~265 tests en 20 suites: núcleo, acciones, percepciones, interp, world, sim, eventos, funciones, expr, lexer, compile, resources, rng, species, value, cli_*).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings.
- `cargo fmt --all --check` → exit 0.
- Determinismo: dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo → salida **byte a byte idéntica** (SHA256 igual; test `test_annex_seed7_sequence_unchanged` + `test_cli_annex_is_deterministic` en verde).
- Cero `println!`, cero `f32`/`f64`, cero dependencias externas en `crates/instinct-core`.
- API CLI intacta (`cargo test -p instinct-cli --test cli_run` → 20 passed).

## Medición de salida de fase (registrada por F4-08 en `PHASE.md#registro-de-ejecución`)

- Tiempo por tick (debug, seed 7, 500 ticks, Walker@5,5): 0.164 → 0.162 ms/tick (no empeora).
- Pico de memoria (working set): 3.94 → 4.10 MB (no empeora significativamente).

## Verificador independiente (verificador-nemotron, sesión fresca)

**Veredicto global: APROBADO CON OBSERVACIONES.** Código y comportamiento correctos; las observaciones son **documentales**:

1. `PHASE.md` tabla de issues: F4-08 figura `todo` (su issue dice `done`).
2. `PHASE.md` registro de ejecución: F4-01, F4-03, F4-04 figuran `pendiente`.
3. Notas de F4-02, F4-06, F4-07 dicen "Commit: pendiente" aunque los commits existen en `git log`.
4. F4-02 AC2/AC4 citan nombres de test que no existen en el repo (`test_turn_state_sleeping_skips_execution`, `test_wait_sets_sleeping_state`, `test_simulation_sleeping_wakes_up`, `test_end_of_tick_ages_sleeping_creature`); los tests equivalentes con otros nombres pasan.
5. El escenario de determinismo de F4-01 (`Hobbit@10,10`) cae fuera del mapa 20×7 (no es regresión: el CLI rechaza la colocación, exit 1 idéntico en ambas corridas).

El juicio final es de la revisión matutina.

## Pendiente para la revisión matutina (cierre documental de F4)

- Corregir las inconsistencias documentales 1-3 arriba (estados en `PHASE.md` y notas de commit).
- Alinear los nombres de test citados en los AC de F4-02 (observación 4) o aceptar los nombres reales.
- Rellenar el "Registro de ejecución" de `PHASE.md` patrón/patrón (columna Medición ya iniciada por F4-08).
- Actualizar `plan/MASTER.md` en el registro de cambios: cierre de F4 (7 implementados + F4-07 descartado, 8/8) y planeación de F5.
- Última suite completa tras los retoques documentales.

## Notas de orquestación

- Sin bloqueos. Ningún issue requirió `rescatista-step5`. Una única reanudación por fallo de infraestructura (F4-03), misma sesión, sin cambio de modelo.
- Artefacto `null` (vacío) borrado durante la sesión.
- Ledger completo: `plan/F4-arquitectura-patrones/LEDGER_NOCHE_F4.md`.

---

## ADDENDA (2026-11-12) — F4-07 reabierto: Camino B (Visitor como único recorrido del AST)

El usuario decidió invertir ahora en el patrón Visitor en lugar de esperar a tener los consumidores, con la restricción de que el intérprete también lo use (evitar dos formas de recorrer el árbol). Reabierto sobre `develop`, commit `822d1c5` + merge.

### Qué cambió
- `expr.rs`: nuevo `pub trait ExprVisitor` con `type Output` y `visit_*` para las 6 variantes (`LiteralInt`, `LiteralText`, `Name`, `Call`, `Unary`, `Binary`). `Expr` **ya no tiene `eval`**: ahora `fn accept(&self, visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>) -> Result<Value, String>`, implementado en los 6 nodos. Los cuerpos de evaluación se movieron tal cual al visitante del intérprete (mensajes de error y short-circuit `and`/`or` preservados).
- `interp.rs`: `pub struct InterpreterVisitor<'a> { ctx }` implementa `ExprVisitor` con `Output = Result<Value, String>`; los 3 sitios de `execute_line` usan `expr.accept(&mut InterpreterVisitor { ctx })`.
- Tests: helpers y llamadas `.eval(&ctx)` → `accept`; nuevo `test_node_counter_counts_all_nodes` (visitor NO-intérprete que demuestra el patrón: `"1 + 2 * 3"` → 5 nodos).
- El parser **no** cambia: construye nodos, no recorre (Visitor no aplica a construcción).
- El issue F4-07 quedó `status: done` con sección "Reapertura (Camino B)" y AC nuevos.

### Verificación (sobre `develop` mergeado)
- `cargo test --workspace` → 0 fallos (~270 tests).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Determinismo intacta: `test_annex_seed7_sequence_unchanged` + `test_cli_annex_is_deterministic` en verde.

### Coste asumido documentado (limitación conocida de Rust)
Como el árbol es `Box<dyn Expr>`, `accept` no puede ser genérico sobre el `Output` del visitante; hoy `Expr::accept` sirve `Output = Result<Value, String>`. Un futuro visitante con otra salida (p. ej. pretty-printer con `String`) o amplía `Expr` con otro punto de entrada o acumula su resultado en campos del propio visitante (como hace `NodeCounter` con `count`). Queda anotado en las notas del issue como extensión futura.

### Pendiente actualizado para revisión matutina
Lo de la lista original sigue vigente (además de esta addenda):
1. `PHASE.md`: la tabla de issues ya quedó corregida por los implementadores salvo confirmar el estado de F4-08/F4-01/F4-03 en la revisión.
2. Confirmar la coherencia del informe con el árbol final.
3. Actualizar `plan/MASTER.md` con el cierre de F4 (8 implementados tras la reapertura) y planeación de F5.
