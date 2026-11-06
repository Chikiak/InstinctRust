---
id: F2-03
title: Errores unificados y cero unwrap/expect sobre datos de entrada
phase: F2
status: done
depends_on: [F2-02]
size: M
---

# F2-03 · Errores unificados y cero `unwrap`/`expect` sobre datos de entrada

## Objetivo

Cerrar los problemas L1, P1, W1 e I1 (y sus duplicados A1, A2, A3 vistos desde el núcleo): un solo
tipo de diagnóstico con archivo y línea en toda la pila, y ni un `unwrap` ni un `expect` ni un
valor por defecto silencioso sobre datos que vienen de un fichero o del mundo.

## Alcance

**Haz**

- **Lexer (L1)**: `lex` recibe también el archivo, `LexError` gana el campo `file` y existe una
  conversión explícita de `LexError` a `Diagnostic`. El sitio de llamada único
  (`expr::parse`, línea 456 de `expr.rs`) usa la conversión en lugar de construir el diagnóstico a
  mano. Actualiza `crates/instinct-core/tests/lexer_tests.rs` a la nueva firma.
- **Parser (P1)**: elimina los tres `unwrap()` sobre `advance()` de `crates/instinct-core/src/expr.rs`
  (líneas 269, 287 y 302) reestructurando `expect`, `parse_or` y `parse_and` para que propaguen
  `Result` sin asumir que `advance` devuelve `Some`. El comportamiento ante tokens inesperados no
  cambia.
- **Mundo (W1)**: `World::draw` devuelve `Result<String, Diagnostic>` en lugar de usar `expect`
  (línea 580 de `world.rs`), y valida los índices de objeto y terreno contra sus catálogos antes de
  indexar (hoy `objects[*idx]` y `terrains[self.terrain[row][column]]`, líneas 583-584, pueden
  panicar si el catálogo recibido no cubre el índice). Actualiza los sitios de llamada y las
  pruebas que usen `draw`.
- **Intérprete (I1)**: `execute_turn` (línea 194 de `interp.rs`) deja de reiniciar en 0 con
  `unwrap_or(0)`: si el programa no tiene etiqueta `start` al final del flujo, devuelve error
  nombrando la etiqueta que falta.
- Pruebas: `draw` con identificador de criatura huérfano en la rejilla, `draw` con catálogo más
  corto que lo que la rejilla referencia, `execute_turn` con un `Program` construido a mano sin
  `start`, y conversión de `LexError` a `Diagnostic` con archivo y línea.
- `cargo test -p instinct-core`, clippy y fmt en verde.

**No hagas**

- No cambies la semántica de los mensajes de error existentes salvo la de `draw` y la de `start`.
- No toques el límite de líneas ni el `pc` (eso es F2-05), ni los catálogos de `Simulation`
  (F2-07), ni la CLI salvo para actualizar la llamada de `draw`.
- No inventes un tipo de error nuevo: `Diagnostic` es el destino.

### Archivos previstos

- `crates/instinct-core/src/lexer.rs` — modificar — `lex` con archivo, `LexError` con `file`, conversión.
- `crates/instinct-core/src/expr.rs` — modificar — sin `unwrap`; uso de la conversión en `parse`.
- `crates/instinct-core/src/world.rs` — modificar — `draw` devuelve `Result` y valida índices.
- `crates/instinct-core/src/interp.rs` — modificar — sin `unwrap_or(0)`.
- `crates/instinct-core/tests/lexer_tests.rs`, `crates/instinct-core/tests/world_tests.rs`, `crates/instinct-cli/src/main.rs` — modificar — sitios de llamada.

## Criterios de aceptación

- [ ] **AC1** `lex` recibe el archivo y `LexError` lleva `file`, `line` y `message`, con conversión explícita a `Diagnostic` que conserva los tres campos. Verif: `test_lex_error_to_diagnostic` | `cargo test -p instinct-core --test lexer_tests` | prueba pasa.
- [ ] **AC2** Cero `unwrap`/`expect` sobre tokens en `expr.rs`; `expect`, `parse_or` y `parse_and` propagan `Result`. Verif: inspección de `expr.rs` + `cargo test -p instinct-core --test expr_tests` | sin `unwrap` y todas las pruebas pasan.
- [ ] **AC3** `World::draw` devuelve `Result<String, Diagnostic>` y ante un identificador huérfano en la rejilla devuelve error, no pánico. Verif: `test_draw_orphan_creature_errors` | `cargo test -p instinct-core --test world_tests` | prueba pasa.
- [ ] **AC4** `draw` no pánico cuando el catálogo de objetos o de terrenos no cubre el índice que guarda la rejilla. Verif: `test_draw_short_catalog_errors` | `cargo test -p instinct-core --test world_tests` | prueba pasa.
- [ ] **AC5** `execute_turn` con un programa sin etiqueta `start` devuelve error que la nombra; nunca reinicia en 0 en silencio. Verif: `test_execute_turn_missing_start_errors` | `cargo test -p instinct-core --test interp_tests` | prueba pasa.
- [ ] **AC6** Los sitios de llamada de `draw` (CLI y pruebas) compilan y se comportan igual con salida válida. Verif: `cargo test --workspace` | todo en verde.
- [ ] **AC7** Suite completa sin regresiones: clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `Diagnostic::new(file, line, message)` en `crates/instinct-core/src/diagnostic.rs`
usado por toda la pila; `ParseError` con `file`/`line`/`message` (líneas 192-208 de `expr.rs`) y su
conversión en `expr::parse`; `LexError` sin archivo (líneas 39-53 de `lexer.rs`); los `unwrap` de
`expr.rs` en 269, 287 y 302; `World::draw` sin `Result` (líneas 570-591 de `world.rs`); el
`unwrap_or(0)` de `interp.rs` línea 194; `compile` ya exige la etiqueta `start` (línea 357 de
`compile.rs`), así que el error de I1 es un invariante defensivo alcanzable solo con un `Program`
construido a mano.
**Contratos que debes respetar**: `Diagnostic` sigue siendo el tipo que cruza módulos; la línea se
cuenta desde 1; el intérprete lee y el mundo escribe; `draw` sigue produciendo exactamente el mismo
ASCII cuando el mundo es consistente.
**Ojo**: al cambiar `draw` a `Result`, la CLI debe decidir qué hace ante el error: abortar la
impresión del mapa con un mensaje por `stderr` es aceptable y se afina en F2-08.

### Leer primero

- `crates/instinct-core/src/diagnostic.rs` — el tipo único de error.
- `crates/instinct-core/src/lexer.rs` — `LexError` y `lex`.
- `crates/instinct-core/src/expr.rs` — `expect`, `parse_or`, `parse_and`, `parse`.
- `crates/instinct-core/src/world.rs` — `draw` y la forma de la rejilla.
- `crates/instinct-core/src/interp.rs` — `execute_turn` y su retorno `(Diagnostic, usize)`.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` secciones 1, 2, 3 y 5 (L1, P1, I1, A1-A3).

## Especificación

La firma nueva del lexer es `lex(text, line, file)` y su error es `LexError { file, line, message }`
con `From<LexError> for Diagnostic` sin pérdida de información; `expr::parse` usa esa conversión.
En el parser, `expect` devuelve el token consumido o error, y `parse_or`/`parse_and` obtienen el
operador del token que ya comprobaron con `peek`, sin volver a llamar a `advance` sobre un `Option`
que hay que desenvolver. `draw` recorre la rejilla y, para cada celda, resuelve la criatura en el
almacén; si no está, error nombrando el identificador; si el índice de objeto o de terreno no cabe
en el catálogo recibido, error nombrando el índice. `execute_turn`, al agotar las líneas, resuelve
`start`: si existe, el turno acaba y el siguiente reanuda ahí; si no existe, error.

## Pasos sugeridos (opcional)

1. Pruebas primero para las cuatro correcciones.
2. Lexer: nuevo campo, nueva firma, conversión, actualiza `expr.rs` y las pruebas.
3. Parser: elimina los `unwrap`-propágando `Result`.
4. Mundo: `draw` a `Result` con validación de índices; actualiza llamadas.
5. Intérprete: error en lugar de `unwrap_or(0)`.
6. `cargo test --workspace` y el gate completo.

## Tests

Nivel: unitario en `lexer_tests.rs`, `world_tests.rs` e `interp_tests.rs`, más integración al
compilar la CLI. Casos como comportamientos: rejilla con id huérfano, catálogo corto, programa sin
`start`, error de lexer con archivo. Qué NO mockear: `World` real y `Diagnostic` real.
Determinismo: no aplica. Datos: mundos construidos a mano y programas mínimos.

## Verificación

- `cargo test -p instinct-core` → todas las pruebas del núcleo pasan.
- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si eliminar un `unwrap` cambia un mensaje de error que una prueba existente fija, ajusta la prueba
y citála en las notas. Decisiones que NO puedes tomar tú: introducir un tipo de error propio
distinto de `Diagnostic`, mantener `draw` devolviendo `String`, o devolver el reinicio silencioso
en 0.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

- Firma final de `lex`: `pub fn lex(text: &str, line: usize, file: &str) -> Result<Vec<SpannedToken>, LexError>`.
- `LexError` lleva `file: String`, `message: String`, `line: usize`; `LexError::new(file, message, line)`.
- Conversión: `impl From<LexError> for Diagnostic` (sin pérdida de `file`/`line`/`message`).
- `expr::parse` usa `lex(text, line, file)` + `Diagnostic::from(err)` vía `match` (evita `redundant_closure` de clippy).
- Eliminados los 3 `unwrap` de `advance` en `expr.rs` (`expect`, `parse_or`, `parse_and`): ahora consumen el token por `pos` o propagan `self.error(...)` sin asumir `Some`.
- `World::draw` → `pub fn draw(&self, terrains: &[Terrain], objects: &[Object]) -> Result<String, Diagnostic>`; valida criatura huérfana (`ok_or_else`), objeto fuera de catálogo y terreno fuera de catálogo (error nombrando el índice), sin `expect` ni índice fuera de rango.
- `execute_turn`: `program.resolve_label("start")` resuelto con `match`; ausencia devuelve `Err((Diagnostic, 0))` nombrando `'start'`; se eliminó `unwrap_or(0)`.
- Call sites actualizados: `crates/instinct-cli/src/main.rs` (2 llamadas a `draw`, aborta con mensaje por stderr y `return 1`); `crates/instinct-core/tests/sim_tests.rs` (4 `.unwrap()` en `draw`).
- Pruebas ajustadas/añadidas: `lexer_tests.rs` (todas las llamadas a `lex` pasan `"test.ins"`; nueva `test_lex_error_to_diagnostic` AC1); `world_tests.rs` (3 llamadas a `draw` con `.unwrap()`; nuevas `test_draw_orphan_creature_errors` AC3 y `test_draw_short_catalog_errors` AC4); `interp_tests.rs` (nueva `test_execute_turn_missing_start_errors` AC5 con `Program` construido a mano).
- Comandos en verde desde la raíz del worktree:
  - `cargo test --workspace`: 150 tests, todo ok, exit 0.
  - `cargo clippy --workspace --all-targets -- -D warnings`: exit 0, sin warnings.
  - `cargo fmt --all --check`: exit 0.
- Sin `#[allow]`; sin dependencias externas; sin `rand`; sin `f32`/`f64`; sin `unwrap`/`expect`/`panic!` sobre datos de usuario o fichero en `instinct-core/src` y `instinct-cli/src` (salvo fallbacks internos documentados).
