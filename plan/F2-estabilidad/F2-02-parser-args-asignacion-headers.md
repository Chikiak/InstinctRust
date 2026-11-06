---
id: F2-02
title: Parser: argumentos propagados, asignación robusta y operadores sin comillas
phase: F2
status: done
depends_on: []
size: M
---

# F2-02 · Parser: argumentos propagados, asignación robusta y operadores sin comillas

## Objetivo

Cerrar los problemas P2, P3, P4 y P5 del parser y la primera mitad de P7: que un error de parseo
de un argumento de acción se reporte en lugar de desaparecer, que la detección de asignación no
acepte nombres inválidos ni confunda `==`, que los operadores nunca se rendericen con comillas, y
que un valor de header inválido produzca exactamente un diagnóstico.

## Alcance

**Haz**

- En `crates/instinct-core/src/compile.rs`, `parse_action_call`: sustituir el
  `.collect::<Result<Vec<_>, _>>().ok()?` (líneas 535-542) por propagación explícita del error de
  cada argumento, de modo que el diagnóstico llegue con su línea y su archivo y la compilación del
  resto del fichero continúe (problema P2).
- En `crates/instinct-core/src/compile.rs`, `looks_like_assignment` (líneas 373-394): rechazar
  nombres cuyo primer carácter no sea una letra o `_` (hoy `is_ascii_alphanumeric` admite los que
  empiezan por dígito) y detectar `==` mirando también el carácter siguiente al `=`, no solo el
  anterior (problema P3).
- En `crates/instinct-core/src/expr.rs`, `operator_text` (líneas 443-449): que devuelva el texto
  pelado del operador para todos los tokens que puede recibir, sin comillas, y documentar que
  `Display for Token` (líneas 464-489) existe solo para mensajes de error (problema P4).
- En `crates/instinct-core/src/compile.rs`: que el valor numérico de una clave de header se parsee
  una sola vez y produzca exactamente un diagnóstico con la línea del valor (problema P5). Los
  sitios implicados son el parseo de `health`, `vision` y `lifespan` (líneas 144-158) y
  `validate_header_value` (líneas 398-424).
- Pruebas en `crates/instinct-core/tests/compile_tests.rs` para la primera mitad de P7: header con
  valores negativos (`health -5`), `lifespan 0`, claves en mayúsculas (`Health 10`), una línea cuyo
  texto contiene la palabra `goto` dentro de una acción o asignación, una acción con argumentos
  vacíos, y varios errores en la misma línea con su orden de diagnóstico.
- `cargo test -p instinct-core`, clippy y fmt en verde.

**No hagas**

- No cambies la firma de `lex` ni `LexError` (F2-03), ni el contrato fail-fast del parser
  (problema P6: se mantiene, sin código nuevo).
- No toques el mundo ni la CLI.
- No conviertas `Binary::op` en enum ni cambies la evaluación de expresiones.

### Archivos previstos

- `crates/instinct-core/src/compile.rs` — modificar — `parse_action_call`, `looks_like_assignment`, ruta de header.
- `crates/instinct-core/src/expr.rs` — modificar — `operator_text` y documentación de `Display`.
- `crates/instinct-core/tests/compile_tests.rs` — modificar — casos nuevos.

## Criterios de aceptación

- [ ] **AC1** Un argumento de acción con error de parseo produce un diagnóstico con la línea del argumento y no se descarta en silencio. Verif: `test_action_arg_error_is_reported` | `cargo test -p instinct-core --test compile_tests` | prueba pasa.
- [ ] **AC2** Una asignación a un nombre que empieza por dígito (`1x = 5`) se rechaza con diagnóstico. Verif: `test_assignment_digit_name_rejected` | `cargo test -p instinct-core --test compile_tests` | prueba pasa.
- [ ] **AC3** Una comparación con `==` no se trata como asignación. Verif: `test_double_equals_not_assignment` | `cargo test -p instinct-core --test compile_tests` | prueba pasa.
- [ ] **AC4** Ningún operador llega con comillas al nodo binario. Verif: `test_binary_op_text_has_no_quotes` | `cargo test -p instinct-core --test compile_tests` | prueba pasa.
- [ ] **AC5** Un valor de header inválido produce exactamente un diagnóstico, con la línea del valor. Verif: `test_header_invalid_value_single_diagnostic` | `cargo test -p instinct-core --test compile_tests` | prueba pasa.
- [ ] **AC6** Casos de P7 cubiertos: header negativo, `lifespan 0`, clave en mayúsculas, texto que contiene `goto`, acción con argumentos vacíos, múltiples errores en una línea. Verif: `cargo test -p instinct-core --test compile_tests` | todas pasan, lista citada en las notas.
- [ ] **AC7** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `compile(text, catalog, perception_catalog, file) -> CompileResult` con diagnósticos
acumulados; `looks_like_assignment` y `parse_assignment` (líneas 373-507); `parse_action_call`
(líneas 512-552); `validate_header_value` (líneas 398-424); `operator_text` y `Display for Token`
en `expr.rs`; `Catalog::resolve(name, arity)` valida nombres y aridades.
**Contratos que debes respetar**: la compilación nunca aborta a la primera línea errónea, solo
acumula diagnósticos; los errores de args hoy descartados deben conservar el orden por línea del
resultado final (`diagnostics.sort_by_key(|d| d.line)`, línea 362); una acción desconocida o con
aridad incorrecta sigue siendo error de compilación.
**Ojo**: al propagar el error de un argumento, el resultado de `parse_action_call` deja de ser
`Option<Line>` con efecto lateral: decide si propagas `Option<Line>` más un diagnóstico explícito o
cambias a un tipo de retorno con `Result`, pero no pierdas el diagnóstico. Decide dentro del issue
y documéntalo en las notas.

### Leer primero

- `crates/instinct-core/src/compile.rs` — `compile`, `looks_like_assignment`, `validate_header_value`, `parse_action_call`.
- `crates/instinct-core/src/expr.rs` — `operator_text`, `Display for Token`, `parse`.
- `crates/instinct-core/tests/compile_tests.rs` — estilo y helpers existentes.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` sección 2 (P1 a P7).

## Especificación

Un argumento de acción que no parsea debe generar un diagnóstico con la línea y el archivo de la
línea de la acción, y el resto del fichero se sigue compilando. La detección de asignación exige un
nombre que empiece por letra o `_` seguido de letras, dígitos o `_`, un `=` simple que no forme
parte de `==` por ninguno de sus dos lados, y una expresión no vacía a la derecha. El texto de un
operador en un nodo binario nunca lleva comillas. El valor numérico de una clave de header se
convierte una sola vez: si no es entero o no cumple el mínimo, un único diagnóstico lo dice; si es
válido, se guarda.

## Pasos sugeridos (opcional)

1. Pruebas primero para P2, P3, P4 y P5; comprueba que fallan.
2. Corrige `parse_action_call` para propagar el error del argumento.
3. Endurece `looks_like_assignment`.
4. Corrige `operator_text` y documenta el contrato de `Display for Token`.
5. Unifica el parseo del valor de header y añade la prueba de "exactamente un diagnóstico".
6. `cargo test -p instinct-core --test compile_tests`.

## Tests

Nivel: unitario, en `crates/instinct-core/tests/compile_tests.rs`. Casos como comportamientos:
argumento roto se reporta, nombre con dígito inicial se rechaza, `==` no es asignación, sin
comillas en operadores, un solo diagnóstico por valor inválido, los casos de header. Qué NO
mockear: `Catalog` real y `PerceptionCatalog` real. Determinismo: no aplica. Datos: textos `.ins`
mínimos escritos a mano.

## Verificación

- `cargo test -p instinct-core --test compile_tests` → todas las pruebas del fichero pasan.
- `cargo test --workspace` → todo en verde (sin regresiones).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si propagar el error del argumento obliga a cambiar la firma de `parse_action_call` de forma que
afecte a `compile`, hazlo dentro del issue y documéntalo. Decisiones que NO puedes tomar tú:
añadir recuperación de errores al parser (P6 queda como está), cambiar los mensajes existentes de
`Catalog`, o tocar la evaluación de expresiones.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama usada: `feat/f2-02-parser-args` (desde `develop`).

Decisión sobre el retorno de `parse_action_call`: se cambió de `Option<Line>` a un enum `ActionOutcome { NotAnAction, Resolved(Line), Failed }` para evitar que un error de argumento (diagnóstico ya pusheado) caiga después en `"unrecognized line"` y duplique el diagnóstico. `compile` hace `continue` en `Resolved` y `Failed`, y solo intenta otras formas de línea en `NotAnAction`.

Cambios de firma (autorizados por el issue): `parse_action_call` retorna ahora `ActionOutcome` en lugar de `Option<Line>`; `validate_header_value` recibe el `Result<i64, ParseIntError>` ya parseado (una sola conversión) en lugar de reparsear.

Archivos modificados:
- `crates/instinct-core/src/compile.rs` — `parse_action_call` (propagación del error de argumento con línea/archivo), `looks_like_assignment` (nombre debe empezar por letra o `_`; `==` rechazado por ambos lados del `=`), ruta de header (parseo único + `validate_header_value` con `Result` previo).
- `crates/instinct-core/src/expr.rs` — `operator_text` devuelve texto pelado (`+`, `==`, `and`, …) sin comillas para todos los tokens; se documentó que `Display for Token` es solo para mensajes de error; se añadió `Expr::operator()` (default `""`, sobrescrito en `Binary`) para permitir leer el operador desde pruebas.

Pruebas nuevas en `crates\instinct-core\tests\compile_tests.rs` (32 totales, 0 fallos):
- `test_action_arg_error_is_reported` (AC1): argumento roto reporta diagnóstico con la línea de la acción.
- `test_action_arg_error_does_not_abort_file`: el resto del fichero sigue compilando tras un argumento malo.
- `test_assignment_digit_name_rejected` (AC2): `1x = 5` se rechaza.
- `test_double_equals_not_assignment` (AC3): `if x == 5 goto start` compila; `x == 5` no se trata como asignación.
- `test_binary_op_text_has_no_quotes` (AC4): operador `+` almacenado sin comillas.
- `test_header_invalid_value_single_diagnostic` (AC5): exactamente un diagnóstico en la línea del valor.
- P7 (AC6): `test_header_negative_value_reports_on_its_line` (`health -5`), `test_header_lifespan_zero_reports_on_its_line` (`lifespan 0`), `test_header_uppercase_key_is_unknown` (`Health 10`), `test_goto_inside_text_is_not_a_statement` (`goto` dentro de texto), `test_action_with_empty_args_reports_arity` (`wait()`), `test_multiple_errors_ordered_by_line_with_header_and_body` (varios errores ordenados por línea).

Pruebas existentes ajustadas: ninguna; todas las de `compile_tests.rs` siguen pasando sin cambios.

Evidencia de comandos (exit 0):
- `cargo test --workspace` → todo verde (32 pruebas en compile_tests; sin FAILED).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings.
- `cargo fmt --all --check` → exit 0.
