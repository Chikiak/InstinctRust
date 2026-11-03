---
id: F0-05
title: Leer percepciones y variables de una criatura
phase: F0
status: done
depends_on: [F0-03, F0-04]
size: M
---

# F0-05 · Leer percepciones y variables de una criatura

## Objetivo
Que una expresión evaluada en nombre de una criatura colocada en el mundo pueda leer sus siete percepciones básicas y sus variables propias. Une las expresiones (F0-03) con el mundo (F0-04) mediante el trait `Perception` y el catálogo.

## Alcance
**Haz**
- Trait `Perception` y registro por nombre en el `Catalog`; los siete tipos `health`, `age`, `lifespan`, `x`, `y`, `tick`, `random`.
- Estado de ejecución de la criatura con sus variables (separadas por criatura).
- Contexto de evaluación real que, dado el mundo y la criatura, resuelve primero las percepciones y después las variables.
- Escritura de variables, rechazando los nombres de percepciones.

**No hagas**
- Funciones `see` y `name`, constantes ni percepciones de cercanía (F2).
- Intérprete, etiquetas ni programa (F0-06 y F0-07).

### Archivos previstos
- `crates/instinct-core/src/perception.rs` — nuevo — trait y siete percepciones.
- `crates/instinct-core/src/actions.rs` — modificar — el `Catalog` también registra percepciones (o un módulo de catálogo nuevo si es más claro).
- `crates/instinct-core/src/world.rs` — modificar — estado de ejecución de cada criatura y el contexto de lectura.

## Criterios de aceptación
- [ ] **AC1** Para una criatura en (4, 2) con vida 10, edad 3, esperanza 50, en un mundo con 7 ticks completados, las siete percepciones valen: `health` 10, `age` 3, `lifespan` 50, `x` 4, `y` 2, `tick` 7, y `random` un entero entre 0 y 99. Verif: `test_perceptions_report_creature_state`
- [ ] **AC2** Evaluar el texto "health * 2 + x" para esa criatura da 24; evaluar "tick" en un mundo recién creado da 0. Verif: `test_expression_reads_perceptions`
- [ ] **AC3** Cada lectura de `random` avanza el generador una vez: dos lecturas en una misma expresión consumen dos valores, y con la misma semilla las secuencias se repiten. Verif: `test_random_consumes_one_draw_per_read`
- [ ] **AC4** Asignar una variable y leerla da su valor, y puede cambiar de entero a texto al reasignarla; leer una variable sin asignar es un error de ejecución que la nombra. Verif: `test_variables_assign_read_and_retype`, `test_unassigned_variable_error`
- [ ] **AC5** Las variables de dos criaturas de la misma especie son independientes. Verif: `test_variables_are_per_creature`
- [ ] **AC6** Asignar a un nombre de percepción falla con un mensaje que lo nombra. Verif: `test_cannot_assign_perception`
- [ ] **AC7** Un test registra una percepción extra definida solo en el test, sin tocar las siete, y una expresión la lee. Verif: `test_catalog_accepts_extra_perception`
- [ ] **AC8** Suite completa, clippy y fmt en verde. Verif: los comandos de Verificación

## Contexto
**Ya existe**: expresiones y `Value` (F0-03) en `crates/instinct-core/src/expr.rs` y `value.rs`; mundo con criaturas, acciones y catálogo (F0-04) en `crates/instinct-core/src/world.rs` y `actions.rs`; `Rng` propiedad del mundo.
**Contratos que debes respetar**: percepciones y vista de solo lectura en `../PHASE.md#contratos-compartidos` (bloque "Percepciones").
**Ojo**: leer `random` modifica el generador aunque el mundo se lea de forma inmutable; el generador necesita mutabilidad interior acotada a él, y solo a él. Los nombres de percepción tienen prioridad sobre las variables, y como están reservados una variable nunca puede coincidir con ellos.
**Rust que se practica**: `Cell` o `RefCell` en un solo campo, separar un préstamo inmutable del mundo del estado mutable de la criatura, `HashMap`, objetos de trait.

### Leer primero
- `crates/instinct-core/src/expr.rs` — contexto de evaluación que debes implementar.
- `crates/instinct-core/src/world.rs` — almacén de criaturas y ticks.
- `crates/instinct-core/src/actions.rs` — cómo se registra y busca por nombre.
- `crates/instinct-core/src/rng.rs` — generador.

## Especificación
- La vista de solo lectura expone datos de la criatura (vida, edad, esperanza, posición) y del mundo (ticks completados) y la lectura de un valor aleatorio.
- Estado de ejecución por criatura en F0: variables. El punto de ejecución y el sueño los añade F0-07.
- La resolución de un nombre devuelve el valor de la percepción si el catálogo la conoce; si no, el de la variable; si no existe, error de ejecución.
- La escritura de variables rechaza cualquier nombre que el catálogo conozca como percepción o como acción.

## Tests
Unitarios sobre un mundo pequeño con criaturas colocadas, evaluando texto de expresiones con el parser de F0-03. No uses contextos falsos aquí: la integración con el mundo real es lo que se prueba. Escribe primero AC3.

## Verificación
- `cargo test -p instinct-core perception` → esperado: AC1 a AC7 en verde.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` → esperado: exit 0.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si no consigues leer `random` sin pedir acceso mutable al mundo completo, para y repórtalo; no conviertas el mundo entero en `RefCell`. No añadas percepciones fuera de las siete.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Añade a `docs/ambiguedades.md` cualquier decisión nueva.
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

AC1 PASS — `test_perceptions_report_creature_state`: salud 10, edad 3, esperanza 50, x 4, y 2, tick 7 y random en 0..=99 leídos del mundo real. (`cargo test -p instinct-core perception` y `cargo test --workspace`)

AC2 PASS — `test_expression_reads_perceptions`: "health * 2 + x" = 24; "tick" en mundo recién creado = 0. (suite en verde)

AC3 PASS — `test_random_consumes_one_draw_per_read`: dos lecturas consumen dos valores, misma semilla repite la secuencia, "random + random" consume dos draws y la siguiente lectura es el tercer valor. (suite en verde)

AC4 PASS — `test_variables_assign_read_and_retype` y `test_unassigned_variable_error`: asignar/leer, cambio de Int a Text, variable no asignada error nombrándola. (suite en verde)

AC5 PASS — `test_variables_are_per_creature`: variables independientes por criatura (usó "energia" porque "x" es percepción reservada). (suite en verde)

AC6 PASS — `test_cannot_assign_perception`: asignar a "health" y a "wait" falla nombrando el nombre. (suite en verde)

AC7 PASS — `test_catalog_accepts_extra_perception`: percepción "bonus" definida solo en el test, leída por una expresión; las siete originales siguen presentes. (suite en verde)

AC8 PASS — `cargo test --workspace` (todo verde), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0, sin warnings), `cargo fmt --all --check` (exit 0).

Cambios de código:
- Nuevo `crates/instinct-core/src/perception.rs`: trait `CreatureView`, trait `Perception` (con `clone_box`), siete percepciones `Health`/`Age`/`Lifespan`/`X`/`Y`/`Tick`/`Random`, `PerceptionCatalog` y `Default` con las siete.
- `crates/instinct-core/src/lib.rs`: registra el módulo `perception`.
- `crates/instinct-core/src/actions.rs`: `Catalog` suma campo `perceptions: PerceptionCatalog`, `register_perception`, `resolve_perception`, `is_action_name`, `is_perception_name`; `Default` registra las siete percepciones.
- `crates/instinct-core/src/world.rs`: `Creature` gana `health`, `age`, `lifespan`, `variables: HashMap<String, Value>`; `World` gana `catalog: Catalog` y `rng: RefCell<Rng>` (mutabilidad interior acotada al generador); `place_creature` inicializa los nuevos campos; `set_rng_seed`, `variable`, `set_variable` (rechaza nombres de acción/percepción); helpers `creature`/`creature_mut`; `CreatureSnapshot` implementa `CreatureView` (`random` avanza `rng` vía `borrow_mut`); `CreatureContext` implementa `expr::EvalContext` resolviendo percepciones primero y variables después.
- Pruebas: `crates/instinct-core/tests/perception_tests.rs` con las 8 pruebas de AC1 a AC7.

Decisiones: los nombres de percepción tienen prioridad sobre variables; escribir a un nombre de percepción o acción es error. `tick` vale 0 en el primer tick. El `Rng` usa `RefCell` solo sobre ese campo, nunca sobre el mundo entero.

Rust practicado: `Cell`/`RefCell` en un solo campo, `HashMap`, objetos de trait, separación de préstamo inmutable del mundo y estado mutable de la criatura.
