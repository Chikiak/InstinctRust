---
id: F0-06
title: Compilar y comprobar una criatura .ins
phase: F0
status: done
depends_on: [F0-05]
size: M
---

# F0-06 · Compilar y comprobar una criatura .ins

## Objetivo
Que un archivo `.ins` se convierta en una especie con su programa compilado, o en una lista de errores con su línea, y que se pueda comprobar desde la terminal con `instinct-cli check`. Es la parte "compilador" del proyecto: cabecera, líneas del cuerpo, etiquetas y análisis de nombres.

## Alcance
**Haz**
- Compilación de un texto `.ins` a `Species` con `Program`, reutilizando el lexer y el parser de expresiones de F0-03.
- Cabecera, formas de línea del cuerpo, tabla de etiquetas y comprobaciones de nombres y aridad contra el catálogo.
- Carga de la carpeta `creatures/` dentro de la carga de recursos, sin que un archivo malo detenga a los demás.
- Subcomando `check` del CLI y el ejemplo `examples/creatures/walker.ins`.

**No hagas**
- Ejecutar el programa (F0-07).
- Funciones `see`/`name`, constantes ni más acciones (F1 y F2).
- El catálogo completo de errores del enunciado (F3): aquí basta con los de la tabla de Especificación.

### Archivos previstos
- `crates/instinct-core/src/compile.rs` — nuevo — cabecera, cuerpo y análisis.
- `crates/instinct-core/src/program.rs` — nuevo — `Program` y sus líneas.
- `crates/instinct-core/src/species.rs` — modificar — la especie guarda su programa.
- `crates/instinct-core/src/resources.rs` — modificar — cargar `creatures/`.
- `crates/instinct-cli/src/main.rs` — modificar — `check`.
- `examples/creatures/walker.ins` — nuevo.
- `crates/instinct-cli/tests/cli_check.rs` y `crates/instinct-cli/tests/fixtures/` — nuevos — archivos inválidos.

## Criterios de aceptación
- [ ] **AC1** `walker.ins` compila: especie `Walker`, facción, vida 10, visión, esperanza 100 y un programa con la etiqueta `start`. Verif: `test_compile_walker_example`
- [ ] **AC2** La cabecera acepta las claves en cualquier orden; la falta de una de las cinco, una clave repetida o desconocida, o un valor que no es entero mayor que 0 (visión al menos 1) generan diagnósticos (la falta, en la línea 1; el resto, en su línea) que nombran la clave. Verif: `test_header_validation`
- [ ] **AC3** El cuerpo admite etiqueta, `goto`, `if` con `goto`, asignación y llamada a acción; las líneas en blanco, los comentarios de línea completa y los finales se ignoran y la sangría no importa. Verif: `test_body_line_forms`
- [ ] **AC4** Un `goto` a una etiqueta inexistente se reporta en la línea del `goto`, una etiqueta duplicada en la línea de la segunda, y la falta de `start:` en la línea 1; cada mensaje nombra la etiqueta. Verif: `test_label_errors`
- [ ] **AC5** Asignar a una percepción, a un nombre de acción o a una palabra reservada, llamar a una acción desconocida o con otro número de argumentos, o usar una función en una expresión, se reporta en su línea nombrando el elemento. Verif: `test_name_and_arity_errors`
- [ ] **AC6** Una línea que no encaja en ninguna forma se reporta con su número; un archivo con varios errores devuelve todos, ordenados por línea. Verif: `test_unrecognized_line_and_multiple_errors`
- [ ] **AC7** `instinct-cli check examples/creatures/walker.ins` imprime `OK` y sale con 0; con un archivo inválido de los fixtures imprime un diagnóstico por línea con la forma `archivo:línea: mensaje` y sale con 1. Verif: `test_cli_check_valid_and_invalid`
- [ ] **AC8** Un `.ins` inválido dentro de `creatures/` no impide cargar los demás ni los mapas. Verif: `test_bad_creature_does_not_stop_loading`
- [ ] **AC9** Suite completa, clippy y fmt en verde. Verif: los comandos de Verificación

## Contexto
**Ya existe**: `Value`, lexer y expresiones (F0-03) en `crates/instinct-core/src/expr.rs` y `lexer.rs`; mundo, `Species`, acciones y catálogo (F0-04); percepciones en `perception.rs` y nombres reservados vía catálogo (F0-05); carga de recursos en `resources.rs` (F0-02).
**Contratos que debes respetar**: "Lenguaje en F0", formato de archivos y reglas de errores en `../PHASE.md#contratos-compartidos`; A5, A10, A11 y A12 en `docs/ambiguedades.md`.
**Ojo**: en la línea `if` el `goto` es palabra reservada, así que separa en el `goto` del final y evalúa lo anterior como expresión. La asignación usa `=` y la comparación `==`: distínguelos en el lexer. Las etiquetas son líneas del programa y cuentan en el límite de 100 que aplica F0-07, por eso `Program` las conserva como líneas. Cada línea de `Program` guarda su número de línea fuente: F0-07 lo necesita para los errores de ejecución.
**Rust que se practica**: acumular errores en un `Vec` en vez de abortar con el primero, `HashMap` para etiquetas, `Rc` para compartir la especie y las acciones, separación entre fases de análisis.

### Leer primero
- `crates/instinct-core/src/expr.rs` — parser de expresiones a reutilizar.
- `crates/instinct-core/src/lexer.rs` — tokens y comentarios.
- `crates/instinct-core/src/actions.rs` — catálogo y aridad.
- `crates/instinct-core/src/perception.rs` — nombres reservados.
- `crates/instinct-core/src/resources.rs` — cómo se cargan las demás carpetas.
- `docs/enunciado.md` — secciones 2.2 a 2.4.

## Especificación
| Error | Línea | El mensaje nombra |
|---|---|---|
| Línea no reconocida | la línea | el texto de la línea |
| Clave de cabecera ausente | 1 | la clave |
| Clave repetida o desconocida | la línea | la clave |
| Valor de cabecera inválido | la línea | la clave |
| Falta `start:` | 1 | `start` |
| Etiqueta duplicada | la segunda | la etiqueta |
| `goto` a etiqueta inexistente | la línea del `goto` | la etiqueta |
| Asignación a nombre reservado | la línea | el nombre |
| Acción desconocida o con aridad distinta | la línea | la acción |
| Función en una expresión | la línea | la función |
| Error de expresión (de F0-03) | la línea | lo que dicen sus mensajes |

- `creature` debe ser la primera línea no vacía ni comentario; la cabecera termina en `start:`.
- Una especie con diagnósticos no se registra, y los diagnósticos se devuelven junto con los recursos válidos.
- `walker.ins`: avanza a la derecha de a una casilla en un bucle, vida 10, esperanza 100, visión 3, facción cualquiera.

## Tests
Unitarios sobre texto fuente en memoria para cada fila de la tabla y sobre `walker.ins` real; integración del CLI con fixtures inválidos. No mockees el catálogo. Escribe primero los casos de AC4 y AC5.

## Verificación
- `cargo test -p instinct-core compile` → esperado: AC1 a AC6 y AC8 en verde.
- `cargo test -p instinct-cli` → esperado: AC7 en verde.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` → esperado: exit 0.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si el parser de F0-03 no sirve para el caso del `if`, repórtalo en vez de duplicar un parser nuevo. Los mensajes exactos no son contrato; lo son la línea y el elemento nombrado.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Añade a `docs/ambiguedades.md` cualquier decisión nueva.
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

AC1 (PASS): `cargo test -p instinct-core --test compile_tests test_compile_walker_example` → walker.ins compiles, species Walker, faction any, health 10, vision 3, lifespan 100, label `start` present.
AC2 (PASS): `cargo test -p instinct-core --test compile_tests test_header_*` → all header validation tests pass (keys in any order, missing key on line 1, repeated key, unknown key, non-integer value, zero health, zero vision).
AC3 (PASS): `cargo test -p instinct-core --test compile_tests test_body_all_line_forms` → label, goto, if-goto, assignment, action call; blank lines, full-line comments, trailing comments, indentation ignored.
AC4 (PASS): `cargo test -p instinct-core --test compile_tests test_goto_to_missing_label_reports_on_goto_line test_duplicate_label_reports_on_second test_missing_start_reports_on_line_1` → goto to missing label on goto line, duplicate label on second, missing start on line 1.
AC5 (PASS): `cargo test -p instinct-core --test compile_tests test_assign_to_perception_reports_on_its_line test_assign_to_action_reports_on_its_line test_assign_to_reserved_word_reports_on_its_line test_unknown_action_reports_on_its_line test_wrong_arity_reports_on_its_line test_function_in_expression_reports_on_its_line` → all name/arity errors report on their line naming the element.
AC6 (PASS): `cargo test -p instinct-core --test compile_tests test_unrecognized_line_reports_its_text test_multiple_errors_ordered_by_line` → unrecognized line names the text; multiple errors ordered by line.
AC7 (PASS): `cargo test -p instinct-cli --test cli_check` → all 5 CLI check tests pass (valid OK exit 0, invalid diagnostics exit 1).
AC8 (PASS): `cargo test -p instinct-core --test resources_tests test_bad_creature_does_not_stop_loading` → bad creature doesn't stop loading others or maps.
AC9 (PASS): `cargo test --workspace` → all pass; `cargo clippy --workspace --all-targets -- -D warnings` → exit 0; `cargo fmt --all --check` → exit 0.

Files created/modified:
- `crates/instinct-core/src/compile.rs` (new) — header validation, body parsing, label table, name/arity checks, `compile()` returning `CompileResult{program, diagnostics, header}`.
- `crates/instinct-core/src/program.rs` (new) — `Program`, `Line` enum, `resolve_label()`, manual `Clone` for `Line` via `Expr::clone_box`.
- `crates/instinct-core/src/species.rs` (modified) — `Species` now stores `faction`, `health`, `vision`, `lifespan`, `program: Option<Rc<Program>>`; `new(name)` with defaults + `with_program(...)`.
- `crates/instinct-core/src/resources.rs` (modified) — `LoadedResources.creatures`, `load_creatures_folder()`, `load_resources()` loads creatures/ last.
- `crates/instinct-core/src/expr.rs` (modified) — `Expr: Debug` trait + `clone_box()` method; manual `Clone` for `Unary`/`Binary`; `clone_box` on all 5 node types.
- `crates/instinct-core/src/lib.rs` (modified) — added `compile` and `program` modules.
- `crates/instinct-cli/src/main.rs` (modified) — `check` subcommand implemented: prints `OK`/diagnostics (`archivo:línea: mensaje`), exit 0/1.
- `examples/creatures/walker.ins` (new) — Walker, faction any, health 10, vision 3, lifespan 100, loops `move(1,0,1); goto start`.
- `crates/instinct-core/tests/compile_tests.rs` (new) — 20 unit tests covering AC1-AC6.
- `crates/instinct-cli/tests/cli_check.rs` (new) — 5 integration tests for AC7.
- `crates/instinct-cli/tests/fixtures/{bad_header,bad_label,bad_action}.ins` (new) — invalid fixtures.
- `crates/instinct-core/tests/resources_tests.rs` (modified) — added `test_bad_creature_does_not_stop_loading` for AC8.

Rust concepts practiced: `Result` accumulation, `HashMap` for labels, `Rc` for shared program, trait object `Box<dyn Expr>` with `clone_box`, separation "compiler reads, world writes".

