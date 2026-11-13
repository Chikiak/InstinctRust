---
id: F5-05
title: Validación acumulativa en `check` y limpieza del compilador
phase: F5
status: todo
depends_on: [F5-02]
size: M
---

# F5-05 · Validación acumulativa en `check` y limpieza del compilador

## Objetivo

Que `check` muestre de una vez todos los errores de un `.ins`: un diagnóstico por argumento mal
formado de una llamada a acción (hoy se corta en el primero) y un diagnóstico por línea con error,
siempre ordenados por línea. Además, eliminar el bloque duplicado de detección de asignación que
arrastra `compile.rs` desde F2.

## Alcance

**Haz**
- `parse_action_call` (en `compile.rs`) recoge los diagnósticos de todos los argumentos mal formados de una llamada antes de darla por fallida, en lugar de devolver el primero.
- Regla de parada declarada: una llamada con argumentos mal formados reporta todos sus argumentos con error y no comprueba además nombre ni aridad de la acción, porque un diagnóstico de aridad sobre un conteo parcial de argumentos confundiría. La línea queda marcada como fallida.
- Confirmar que un `.ins` con errores en varias líneas produce un diagnóstico por línea, ordenados por línea ascendente, y fijarlo con prueba.
- Eliminar el bloque duplicado de detección de asignación en `compile.rs`: la segunda copia está muerta desde que la primera consume las asignaciones antes de `goto` e `if`.
- Dejar documentado en las Notas de implementación que el lexer mantiene fail-fast (contrato L6 de F2) y que la acumulación se aplicó donde aporta.

**No hagas**
- No abras el contrato del lexer: sigue fallando en el primer error (decisión 4 de la fase).
- No cambies mensajes existentes ni el orden de diagnósticos por línea.
- No toques la CLI ni los recursos.

### Archivos previstos
- `crates/instinct-core/src/compile.rs` — modificar — acumulación en `parse_action_call` y eliminación del bloque duplicado de detección de asignación.
- `crates/instinct-core/tests/compile_tests.rs` — modificar — casos de acumulación de argumentos y de errores por línea.
- `crates/instinct-cli/tests/cli_check_errors.rs` — nuevo en F5-02; modificar solo si hace falta un caso de acumulación que la batería aún no cubra.

## Criterios de aceptación

- [ ] **AC1** Una llamada a acción con dos argumentos mal formados produce dos diagnósticos, uno por argumento, y ningún diagnóstico de aridad sobre la misma línea. Verif: `test_action_call_collects_all_argument_errors` | `cargo test -p instinct-core --test compile_tests`.
- [ ] **AC2** Una llamada con nombre desconocido o aridad incorrecta sigue produciendo un único diagnóstico que nombra la acción y las aridades esperada y recibida. Verif: `test_unknown_action_and_wrong_arity_still_single_diagnostic` | `cargo test -p instinct-core --test compile_tests`.
- [ ] **AC3** Un `.ins` con errores en tres líneas distintas produce tres diagnósticos ordenados por línea ascendente al ejecutar `check`. Verif: `test_check_reports_all_line_errors` | `cargo test -p instinct-cli --test cli_check_errors`.
- [ ] **AC4** `compile.rs` detecta la asignación una sola vez por línea: el segundo bloque de detección ya no existe. Verif: revisión del archivo más `cargo clippy --workspace --all-targets -- -D warnings` en verde sin código muerto nuevo.
- [ ] **AC5** Los tests existentes de `compile_tests.rs` siguen pasando sin cambios, incluido `test_multiple_errors_ordered_by_line_with_header_and_body`. Verif: `cargo test -p instinct-core --test compile_tests`.
- [ ] **AC6** La batería de F5-02 sigue íntegra: todos sus casos dan el mismo mensaje y línea. Verif: `cargo test -p instinct-cli --test cli_check_errors`.
- [ ] **AC7** Suite completa, clippy y fmt en verde. Verif: `cargo test --workspace` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo fmt --all --check`.

## Contexto

**Ya existe**: `compile` acumula diagnósticos por línea y los ordena por número de línea; `parse_action_call` corta en el primer argumento con error; la resolución de acción valida nombre y aridad con un solo diagnóstico; la batería de F5-02 con sus casos de `check`.

**Contratos que debes respetar**: acumulación sin cambiar mensajes; un error de parseo de argumento nunca se descarta en silencio (P2 cerrado en F2); el `Display` de `Diagnostic` no cambia.

**Ojo**: la decisión de no añadir diagnóstico de aridad cuando hay argumentos mal formados es deliberada, por el conteo parcial. Si la implementación actual ya emitiera ambos, quédate con el diagnóstico de aridad único y repórtalo en Notas: significa que el conteo se hace antes de parsear los argumentos.

### Leer primero
- `crates/instinct-core/src/compile.rs` — `parse_action_call`, `split_action_args` y el flujo de líneas del cuerpo.
- `crates/instinct-core/tests/compile_tests.rs` — `test_unknown_action_reports_on_its_line` y `test_multiple_errors_ordered_by_line_with_header_and_body` (estilo y cobertura actual).
- la suite de la batería que crea F5-02 (casos de `check` por archivo inválido) — lo que no debe regresionar.

## Especificación

| Entrada | Salida esperada |
|---|---|
| Llamada a acción con dos argumentos, ambos mal formados | dos diagnósticos, uno por argumento, en la misma línea; sin diagnóstico de aridad |
| Llamada con aridad incorrecta | un diagnóstico que nombra la acción y las aridades |
| Llamada con nombre de acción desconocido | un diagnóstico de acción desconocida |
| Tres líneas con error cada una | tres diagnósticos ordenados por línea |

## Tests

Nivel: unitario en `compile_tests.rs` (compilación en memoria con catálogo real) y un caso de
integración en la suite de la batería. Escribe primero los tests que fallan: hoy la llamada con dos
argumentos mal formados produce un solo diagnóstico.

## Verificación

- `cargo test -p instinct-core --test compile_tests` → esperado: pasan los casos nuevos y los existentes sin cambios.
- `cargo test -p instinct-cli --test cli_check_errors` → esperado: batería íntegra.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → esperado: exit 0.
- `cargo fmt --all --check` → esperado: exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde.

## Si te bloqueas

Si acumular errores en `parse_action_call` cambia el número de diagnósticos de algún caso de la
batería o de F3, repórtalo: la batería es contrato y su `README` manda. Decisiones que no puedes
tomar: abrir el fail-fast del lexer, cambiar mensajes, tocar la CLI.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: comandos ejecutados y resultado, AC por AC.

### Notas de implementación

<!-- Rama, commit, comandos ejecutados y resultado, AC por AC, desviaciones. -->
