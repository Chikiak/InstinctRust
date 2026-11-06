---
id: F2-05
title: Intérprete: pc válido y límite de líneas sin off-by-one
phase: F2
status: done
depends_on: [F2-03]
size: M
---

# F2-05 · Intérprete: `pc` válido y límite de líneas sin off-by-one

## Objetivo

Cerrar los problemas I2 e I4 y la cobertura I5 del intérprete: que el límite de 100 líneas cuente
líneas ejecutadas y deje `pc` en una línea válida, que un salto a un índice inexistente sea error y
no un pánico, y que los caminos del intérprete que hoy no tienen prueba la tengan.

## Alcance

**Haz**

- En `crates/instinct-core/src/interp.rs`, corregir el off-by-one del límite: comprobar el límite
  antes de contar la línea como ejecutada, de modo que un turno ejecute como máximo 100 líneas y
  `pc` quede apuntando a una línea válida (la siguiente no ejecutada). Documentar el contrato.
- Validar el destino de `goto`/`if goto` contra `program.lines.len()` antes de asignarlo a `pc`
  (I4): si el destino queda fuera de rango —alcanzable solo si el programa se muta después de
  compilar—, devolver error de ejecución nombrando la etiqueta.
- Pruebas en `crates/instinct-core/tests/interp_tests.rs` para I5: `IfGoto` verdadero, `IfGoto`
  falso, `IfGoto` con error de evaluación, `goto` a etiqueta desconocida, error al evaluar
  argumentos de una acción, varias acciones `say` en un mismo turno, overflow aritmético
  (`i64::MAX + 1`), resto por cero y negación de `i64::MIN`.
- `cargo test -p instinct-core`, clippy y fmt en verde.

**No hagas**

- No modeles el estado de sueño ni muevas `sleep_turns`: el patrón State es F4-02 (problema I3).
- No cambies el contrato "una acción pendiente por turno" ni "el mundo aplica las acciones".
- No emitas eventos desde el intérprete: el bus de eventos es F4-01 (problema I6).

### Archivos previstos

- `crates/instinct-core/src/interp.rs` — modificar — `execute_turn`, `execute_line`.
- `crates/instinct-core/tests/interp_tests.rs` — modificar — cobertura I5.

## Criterios de aceptación

- [ ] **AC1** Un programa con más de 100 líneas ejecuta exactamente 100 por turno y deja `pc` en una línea válida. Verif: `test_line_limit_executes_100_and_pc_valid` | `cargo test -p instinct-core --test interp_tests` | prueba pasa.
- [ ] **AC2** Un `pc` que quedaría fuera de rango tras un salto produce error de ejecución, nunca un índice fuera de rango. Verif: `test_goto_out_of_range_errors` | `cargo test -p instinct-core --test interp_tests` | prueba pasa.
- [ ] **AC3** La cobertura I5 existe: `IfGoto` verdadero/falso/con error, `goto` a etiqueta desconocida, error evaluando argumentos, varias `say` en un turno. Verif: `cargo test -p instinct-core --test interp_tests` | todas pasan, lista citada en las notas.
- [ ] **AC4** Los casos aritméticos son errores de ejecución y no pánicos ni en release ni en debug: overflow, resto por cero, negación de `i64::MIN`. Verif: `test_arithmetic_extremes_are_runtime_errors` | `cargo test -p instinct-core --test interp_tests` | prueba pasa.
- [ ] **AC5** El contrato del límite queda documentado en `interp.rs` (líneas ejecutadas, no intentos). Verif: lectura del doc comment.
- [ ] **AC6** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `execute_turn` con `lines_executed` incrementado antes de la comprobación
(líneas 165-178 de `interp.rs`), `execute_line` con las cinco variantes de `Line`,
`CreatureTurnState { pc, sleep_turns, variables }`, `PendingAction` y el retorno de error
`(Diagnostic, usize)`; la resolución de etiquetas en `Program::resolve_label`; las pruebas
actuales de `interp_tests.rs`, que hoy fijan `sleep_turns` a mano desde fuera (I3, que aquí no se
toca).
**Contratos que debes respetar**: el intérprete lee y devuelve como mucho una acción pendiente;
`sleep_turns > 0` salta el turno y decrementa; el mundo muere por error de ejecución a través de
`die_by_error` en `sim.rs`.
**Ojo**: cambiar el límite cambia cuántas líneas ejecuta un programa sin acción por turno; las
pruebas existentes que dependan del corte deben ajustarse y citarse en las notas.

### Leer primero

- `crates/instinct-core/src/interp.rs` entero (203 líneas).
- `crates/instinct-core/src/program.rs` — `resolve_label` y `Line`.
- `crates/instinct-core/tests/interp_tests.rs` — estilo y contexto falso de evaluación.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` sección 3 (I1 a I6).

## Especificación

Un turno ejecuta líneas hasta encontrar una acción pendiente, agotar el programa o alcanzar 100
líneas ejecutadas. Al alcanzar el límite, el turno acaba sin acción y `pc` queda en la primera
línea no ejecutada. Un salto cuyo destino no existe en `lines` es error de ejecución con el mensaje
de etiqueta desconocida que ya existe.

## Pasos sugeridos (opcional)

1. Escribe la prueba del límite (más de 100 líneas) y comprueba el fallo actual.
2. Mueve la comprobación del límite antes del incremento y deja `pc` en una línea válida.
3. Añade la validación de rango del destino del salto.
4. Completa la cobertura I5, incluidos los casos aritméticos.
5. `cargo test -p instinct-core --test interp_tests` y el gate completo.

## Tests

Nivel: unitario, en `interp_tests.rs`. Casos como comportamientos: 100 líneas ejecutadas y `pc`
válido, salto fuera de rango, `IfGoto` en sus tres caminos, error de argumentos, varias `say`, y
los tres extremos aritméticos. Qué NO mockear: el `Program` real compilado y un `EvalContext` de
prueba sencillo. Determinismo: no aplica (no hay `random` aquí). Datos: textos `.ins` mínimos con
etiquetas.

## Verificación

- `cargo test -p instinct-core --test interp_tests` → todas las pruebas del fichero pasan.
- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si el invariante de `pc` requiere un cambio en `sim.rs` (por ejemplo, al restaurar el estado),
hazlo mínimo y documéntalo. Decisiones que NO puedes tomar tú: cambiar el valor del límite de 100,
mover el sueño al intérprete (es F4-02) o emitir eventos desde aquí (es F4-01).

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

- AC1 PASS: `test_line_limit_executes_100_and_pc_valid` — programa de 152 líneas; el turno se detiene en `pc == 100` (primera línea no ejecutada, válida) con exactamente 100 líneas ejecutadas (`n == 4753`, 98 sumas). `cargo test -p instinct-core --test interp_tests` → ok.
- AC2 PASS: `test_goto_out_of_range_errors` y `test_if_goto_out_of_range_errors` — destino fuera de rango devuelve `Err` nombrando la etiqueta (línea 2), sin pánico ni índice fuera de rango; `pc` se conserva en 1.
- AC3 PASS: cobertura I5 — `test_if_goto_true_jumps`, `test_if_goto_false_falls_through`, `test_if_goto_eval_error_is_runtime_error`, `test_goto_unknown_label_is_runtime_error`, `test_action_argument_eval_error_is_runtime_error`, `test_multiple_says_in_one_turn`. `cargo test -p instinct-core --test interp_tests` → 17 passed, 0 failed.
- AC4 PASS: `test_arithmetic_extremes_are_runtime_errors` — overflow (`i64::MAX + 1`), resto por cero (`7 % 0`), negación de `i64::MIN` (`m = i64::MIN`; `n = -m`) → errores de ejecución con línea (8), sin pánico en debug ni en release (`--test interp_tests --release` → ok).
- AC5 PASS: contrato documentado en el doc comment de `LINE_LIMIT` y `execute_turn` en `crates/instinct-core/src/interp.rs`: el límite cuenta líneas ejecutadas, no intentos, y `pc` queda en una línea válida.
- AC6 PASS: `cargo test --workspace` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all --check` exit 0.
- Pruebas existentes ajustadas: ninguna. `test_line_limit_ends_turn` sigue pasando (`pc == 2`); no se modificó ninguna prueba existente, solo se añadieron 10 en `crates/instinct-core/tests/interp_tests.rs`.
- Firmas cambiadas: ninguna. Se añadió la función privada `jump_target` en `crates/instinct-core/src/interp.rs`; `execute_line` y `execute_turn` conservan sus firmas. Sin cambios fuera de `interp.rs` y `interp_tests.rs` salvo este archivo y `PHASE.md`.
- Rama: `feat/f2-05-interp-pc-limite`; `git merge develop` → "Already up to date", sin conflictos.
