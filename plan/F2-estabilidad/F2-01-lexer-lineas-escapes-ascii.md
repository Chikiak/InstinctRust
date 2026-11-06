---
id: F2-01
title: Lexer: contrato de líneas, escapes en textos e identificadores ASCII
phase: F2
status: done
depends_on: []
size: M
---

# F2-01 · Lexer: contrato de líneas, escapes en textos e identificadores ASCII

## Objetivo

Cerrar los problemas L2, L3, L4, L5 y L6 del lexer: que un salto de línea cuente exactamente una
vez, que los textos admitan escapes, que el subconjunto de caracteres sea ASCII por decisión
explícita, y que cada caso del lexer tenga prueba. Es el issue más independiente de la fase: solo
toca `lexer.rs` y sus pruebas.

## Alcance

**Haz**

- Normalizar los saltos de línea en `crates/instinct-core/src/lexer.rs`: `\r\n`, `\n` y `\r`
  aislados incrementan el contador de línea exactamente una vez (hoy `\r` y `\n` se tratan por
  separado, línea 83-86, y un CRLF cuenta dos).
- Soportar escapes dentro de los textos delimitados por `"`: `\"` produce una comilla y `\\` una
  barra invertida. Cualquier otra secuencia con barra se mantiene literal tal cual. Un texto sin
  comilla de cierre sigue siendo error `unterminated text literal` con la línea correcta.
- Documentar en el módulo y en la función `lex` que el contrato es fail-fast: el primer error
  aborta y no se acumulan diagnósticos (problema L6, se cierra por contrato).
- Documentar que los identificadores y los dígitos son ASCII: los `is_ascii_alphabetic`,
  `is_ascii_alphanumeric` y `is_ascii_digit` se mantienen y un carácter no ASCII cae en el error
  `unknown character` que ya existe (problema L5, decisión tomada por el plan).
- Ampliar `crates/instinct-core/tests/lexer_tests.rs` con los casos que faltan (problema L4):
  `Number`, `Name`, operadores `+ - * / %`, paréntesis, coma, operadores de dos caracteres
  `== != <= >=`, palabras clave `and`/`or`/`not`, entero fuera de rango, texto vacío `""`, texto
  que contiene `#`, texto con salto de línea interior (error), línea vacía (cero tokens), texto
  de varias líneas con números de línea correctos, CRLF como un solo salto, y los escapes nuevos.
- `cargo test -p instinct-core`, clippy y fmt en verde.

**No hagas**

- No cambies la firma de `lex` ni el tipo `LexError`: eso es F2-03.
- No toques `expr.rs`, `compile.rs` ni el intérprete.
- No añadas soporte Unicode ni secuencias de escape adicionales (`\n`, `\t`).
- No cambies el significado del comentario `#` ni de los textos: fuera de un texto empieza un
  comentario, dentro de un texto es un carácter ordinario.

### Archivos previstos

- `crates/instinct-core/src/lexer.rs` — modificar — saltos de línea, escapes, documentación del contrato.
- `crates/instinct-core/tests/lexer_tests.rs` — modificar — cobertura L4 completa.

## Criterios de aceptación

- [ ] **AC1** Un texto con `\r\n` produce tokens cuyas líneas avanzan en 1 por salto, no en 2. Verif: `test_lex_crlf_counts_one_line` | `cargo test -p instinct-core --test lexer_tests` | prueba pasa.
- [ ] **AC2** Dentro de un texto, `\"` y `\\` se interpretan como comilla y barra; una barra seguida de otro carácter se mantiene literal. Verif: `test_lex_text_escapes` | `cargo test -p instinct-core --test lexer_tests` | prueba pasa.
- [ ] **AC3** Un texto sin comilla de cierre sigue siendo error `unterminated text literal` con la línea correcta. Verif: `test_lex_unterminated_text` | `cargo test -p instinct-core --test lexer_tests` | prueba pasa.
- [ ] **AC4** Un carácter no ASCII dentro de un nombre produce `unknown character` (los identificadores siguen siendo ASCII). Verif: `test_lex_non_ascii_name_errors` | `cargo test -p instinct-core --test lexer_tests` | prueba pasa.
- [ ] **AC5** Los casos de L4 existen como pruebas: entero fuera de rango, texto vacío, texto con `#`, texto con salto interior, línea vacía, varias líneas, operadores simples y de dos caracteres, palabras clave, paréntesis y coma. Verif: `cargo test -p instinct-core --test lexer_tests` | todas pasan, lista citada en las notas.
- [ ] **AC6** El contrato fail-fast y el subconjunto ASCII quedan documentados en el doc comment del módulo y de `lex`. Verif: lectura de `lexer.rs` | aparece en la documentación.
- [ ] **AC7** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `Token`, `SpannedToken`, `LexError` y `lex(text, line)` en
`crates/instinct-core/src/lexer.rs`; la rama de saltos en las líneas 83-86; la rama de textos en
las líneas 120-145; la rama de identificadores en las líneas 148-166; las pruebas actuales en
`crates/instinct-core/tests/lexer_tests.rs`.
**Contratos que debes respetar**: `lex` devuelve `Result<Vec<SpannedToken>, LexError>` y falla en
el primer error; `#` fuera de un texto empieza comentario; los números fuera de rango de `i64` son
error; un texto no puede contener un salto de línea sin cerrar.
**Ojo**: los escapes cambian el contenido de los textos, así que cualquier prueba que afirme el
valor de un texto con barra debe revisarse; hoy no hay ninguna porque los escapes no existían.

### Leer primero

- `crates/instinct-core/src/lexer.rs` entero (215 líneas).
- `crates/instinct-core/tests/lexer_tests.rs` — estilo de pruebas existente.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` sección 1 (problemas L1 a L6).

## Especificación

Recorrido de caracteres con `chars()` sobre el texto. Un salto de línea es una unidad: si el
carácter actual es `\r` y el siguiente es `\n`, se consumen los dos y la línea avanza una vez; si
es `\r` solo o `\n` solo, se consume uno y la línea avanza una vez. Los espacios y tabuladores
siguen separando tokens sin tocar la línea. Dentro de un texto, al encontrar `\` se mira el
carácter siguiente: si es `"` se emite comilla, si es `\` se emite barra, en otro caso la barra y
el carácter se emiten tal cual. El texto no puede contener un salto de línea sin cerrarse. El
resto del lexer no cambia.

## Pasos sugeridos (opcional)

1. Escribe las pruebas nuevas primero (TDD) y comprueba que fallan por el motivo esperado.
2. Unifica el tratamiento de `\r`, `\n` y `\r\n` en una sola rama.
3. Añade el manejo de escapes en la rama de textos.
4. Completa la cobertura L4 y ejecuta `cargo test -p instinct-core --test lexer_tests`.

## Tests

Nivel: unitario, en `crates/instinct-core/tests/lexer_tests.rs`. Casos como comportamientos: un
CRLF cuenta una línea, un texto con `\"` conserva la comilla, un texto sin cerrar da error, un
nombre con carácter no ASCII da error, una línea vacía no produce tokens, un texto de varias
líneas numera bien. Qué NO mockear: nada, `lex` es función pura. Determinismo: no aplica. Datos:
literales escritos a mano en cada prueba.

## Verificación

- `cargo test -p instinct-core --test lexer_tests` → todas las pruebas del fichero pasan.
- `cargo test --workspace` → todo en verde (sin regresiones).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si el contrato de `lex` (una línea por lectura) contradice algún caso real de `compile.rs`, para y
repórtalo en "Notas de implementación" en lugar de cambiar la firma: cambiarla es decisión de
F2-03. Decisiones que NO puedes tomar tú: añadir escapes distintos de `\"` y `\\`, abrir Unicode, o
acumular errores en el lexer.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: `feat/f2-01-lexer-contrato`, creada desde `develop` (38930a2); `git merge develop` → fast-forward a c9129ba sin conflictos en archivos de alcance (lexer.rs, lexer_tests.rs).

Cambios (solo alcance permitido):
- `crates/instinct-core/src/lexer.rs`:
  - Rama de saltos unificada: `\r\n` consume ambas unidades y suma 1 a la línea; `\r` solo y `\n` solo suman 1 (fue 2 para CRLF). Idéntico dentro de textos para el error de unterminado.
  - Textos delimitados por `"` con escapes `\"` → `"` y `\\` → `\`; barra + cualquier otro carácter se conserva literal. Sin comilla de cierre → `unterminated text literal` con la línea del salto/texto actual.
  - Doc comment del módulo y de `lex`: contrato fail-fast (primer error detiene, sin acumulación), alfabeto ASCII (`is_ascii_*`, no ASCII = `unknown character`), conteo de saltos de línea.
  - Sin cambio de firma de `lex` ni de `LexError` (reservado a F2-03).
- `crates\instinct-core\tests\lexer_tests.rs`: 4 pruebas previas conservadas + 17 nuevas (21 en total). Lista L4 cubierta: `test_lex_number`, `test_lex_integer_out_of_range`, `test_lex_name`, `test_lex_keywords`, `test_lex_operators`, `test_lex_two_char_operators`, `test_lex_parens_and_comma`, `test_lex_empty_text`, `test_lex_text_with_hash`, `test_lex_crlf_counts_one_line`, `test_lex_lone_cr_counts_one_line`, `test_lex_multiple_lines_numbered_correctly`, `test_lex_empty_line_no_tokens`, `test_lex_text_escapes`, `test_lex_unterminated_text`, `test_lex_text_with_interior_newline_is_error`, `test_lex_non_ascii_name_errors`.

Evidencia AC por AC:
- AC1 PASS — `cargo test -p instinct-core --test lexer_tests`: `test_lex_crlf_counts_one_line` ok (líneas [1, 2], no [1, 3]).
- AC2 PASS — `test_lex_text_escapes` ok: `\"` y `\\` interpretados, `\d` literal.
- AC3 PASS — `test_lex_unterminated_text` y `test_lex_text_with_interior_newline_is_error` ok: `unterminated text literal`, línea correcta (2 tras salto interior).
- AC4 PASS — `test_lex_non_ascii_name_errors` ok: `naïve` → `unknown character` con `ï`, línea 1.
- AC5 PASS — 21 pruebas del fichero en verde; casos L4 todos citados arriba.
- AC6 PASS — doc comment del módulo y de `lex` documentan fail-fast y ASCII (lectura de `lexer.rs`).
- AC7 PASS — `cargo test --workspace`: 0 failed en todas las suites; `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, sin warnings; `cargo fmt --all --check`: exit 0.

Pruebas existentes ajustadas: ninguna (ninguna prueba afirmaba texto con barra; las 4 previas pasan sin cambios).
Firmas cambiadas: ninguna (se respeta `lex(text, line)` y `LexError` sin archivo; F2-03 tocará eso).
