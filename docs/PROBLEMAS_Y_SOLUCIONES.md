# Problemas Principales del Proyecto InstinctRust

## Introducción y Alcance

Este documento recopila los principales problemas del proyecto **InstinctRust** (intérprete y simulador del lenguaje Instinct en Rust), separados por parte del sistema: **lexer, parser/AST, intérprete, mundo/simulación, arquitectura/núcleo y CLI**.

Para cada problema se indica **cómo se resolverá** (patrón de diseño específico, corrección de implementación, feature nuevo o pruebas), sin opciones alternativas ni recomendadas: **se resuelve así y punto**. Está pensado para alimentar directamente el plan de implementación.

La investigación fue realizada por seis subagentes `explore` en paralelo, cada uno cubriendo un área distinta, sobre la base de los archivos fuente en `crates/instinct-core/src/` y `crates/instinct-cli/src/main.rs`, junto con sus suites de pruebas.

### Partes del proyecto investigadas

| Parte | Archivos principales | Enfoque |
|-------|----------------------|---------|
| Lexer | `src/lexer.rs`, `tests/lexer_tests.rs` | Tokenización de entrada de texto |
| Parser y AST | `src/expr.rs`, `src/compile.rs`, `tests/expr_tests.rs`, `tests/compile_tests.rs` | Análisis sintáctico, AST, compilación de programas `.ins` |
| Intérprete | `src/interp.rs`, `tests/interp_tests.rs` | Ejecución por turno, `pc`, `sleep_turns`, variables |
| Mundo y simulación | `src/world.rs`, `src/sim.rs`, `src/species.rs`, `src/resources.rs`, `src/perception.rs`, `src/actions.rs` y sus tests | Rejilla, criaturas, especies, recursos, percepciones, acciones, ticks |
| Arquitectura y núcleo | `src/lib.rs`, `src/program.rs`, `src/value.rs`, `src/diagnostic.rs`, `src/event.rs` y sus tests | Tipos base, diagnóstico, eventos, representación de programa |
| CLI | `crates/instinct-cli/src/main.rs`, `tests/cli_*.rs` | Comandos `run` y `check`, parsing de argumentos, salida |

---

## 1. Lexer (`src/lexer.rs`)

### Problemas detectados

| # | Problema | Evidencia |
|---|----------|-----------|
| L1 | **No se integra con `Diagnostic`**: el lexer devuelve su propio `LexError { message, line }` sin `file` ni conversión al sistema de diagnósticos usado por parser e intérprete. | `lexer.rs:41-53`; `diagnostic.rs:7-11` |
| L2 | **Semántica de línea ambigua**: la firma recibe `line: usize` pero el cuerpo incrementa `current_line` al encontrar `\r` o `\n`; con `\r\n` se incrementa dos veces. | `lexer.rs:68, 83-86, 107, 142, 157, 161, 181, 209` |
| L3 | **Falta de escapes en textos**: los literales `"` no soportan `\"` ni `\\`; una comilla interna termina el literal. | `lexer.rs:120-145` |
| L4 | **Cobertura de pruebas débil**: faltan pruebas para `Number`, `Name`, operadores `+ - * / %`, paréntesis, coma, operadores de dos caracteres `== != <= >=`, keywords `and/or/not`, entero fuera de rango, texto vacío, newline interior, línea vacía, múltiples líneas. | `lexer_tests.rs:6-31` |
| L5 | **Solo ASCII**: identificadores y dígitos requieren `is_ascii_*`; caracteres Unicode se reportan como `unknown character`. | `lexer.rs:148-166` |
| L6 | **Sin recuperación de errores**: el primer error aborta; no se acumulan diagnósticos. | `lexer.rs:68-209` |

### Resolución de problemas

| ID | Tipo de resolución | Qué se hace exactamente |
|----|-------------------|-------------------------|
| L1 | **Patrón de diseño: Error Chain / Envoltura de errores (Diagnostic unificado)** | Añadir campo `file: String` a `LexError` (o unificar directamente bajo `Diagnostic`) y proporcionar conversión a `Diagnostic { file, line, message }` para que toda la pila use un único tipo de error con contexto de archivo. |
| L2 | **Corrección de implementación** | Normalizar el manejo de `\r`, `\n` y `\r\n` para que la línea se incremente exactamente una vez por salto de línea; definir si `lex` opera sobre una sola línea o multilínea y documentar el contrato. |
| L3 | **Feature nuevo** | Soportar secuencias de escape `\"` y `\\` dentro de literales de texto delimitados por `"`. |
| L4 | **Pruebas** | Agregar pruebas unitarias en `lexer_tests.rs` para `Number`, `Name`, operadores simples, paréntesis, coma, operadores de dos caracteres, keywords, entero fuera de rango, texto vacío, texto con newline interior, línea vacía y múltiples líneas. |
| L5 | **Corrección de implementación** | Decidir si el lexer soporta identificadores Unicode o se mantiene estrictamente ASCII; documentar la decisión y ajustar los checks `is_ascii_*` en consecuencia. |
| L6 | **Corrección de implementación** | Mantener fail-fast en el lexer (primer error aborta) como contrato actual; si se requiere acumulación en el futuro, se evaluará como cambio de API separado. No se aplica acumulativa aquí sin romper el contrato de `lex()`. |

---

## 2. Parser y AST (`src/expr.rs`, `src/compile.rs`)

### Problemas detectados

| # | Problema | Evidencia |
|---|----------|-----------|
| P1 | **`unwrap` en `Parser::expect` y en `advance`**: `self.advance().unwrap()` asume invariante de `peek` y puede paniquear con tokens inesperados. | `expr.rs:267-281`, `287`, `302` |
| P2 | **Pérdida silenciosa de diagnósticos en argumentos de acción**: `.collect::<Result<_,_>>().ok()?` descarta el error de parseo de un argumento sin reportarlo. | `compile.rs:532-542` |
| P3 | **Detección de asignación frágil** (`looks_like_assignment` acepta nombres que empiezan por dígito y no evita `==`). | `compile.rs:373-394`, `208`, `288` |
| P4 | **Operadores como texto con comillas**: `operator_text` devuelve `token.to_string()` que para `+` sería `"' +'"`; trampa si se reutiliza fuera de `or/and`. | `expr.rs:443-449`, `464-489` |
| P5 | **Validación de header duplicada**: `validate_header_value` se ejecuta aunque el parseo a `i64` ya falló, generando dos diagnósticos por el mismo valor. | `compile.rs:398-424` |
| P6 | **Parser falla duro, sin recuperación**: ante el primer error de parseo no intenta sincronizar ni acumula múltiples errores. | `expr.rs:232-238`, `240-246` |
| P7 | **Cobertura de pruebas insuficiente**: faltan pruebas para header con valores negativos, `lifespan` cero, claves mayúsculas, `if` con `goto` dentro de texto, acción con args vacíos, múltiples errores en una línea, determinismo de `HashMap` de labels. | `compile_tests.rs:28-296` |

### Resolución de problemas

| ID | Tipo de resolución | Qué se hace exactamente |
|----|-------------------|-------------------------|
| P1 | **Patrón de diseño: Result / Option idiomático** | Reemplazar `self.advance().unwrap()` y `unwrap` en `Parser::expect` por propagación de `Result<Token, ParseError>`; eliminar todos los `unwrap` sobre datos del parser. |
| P2 | **Corrección de implementación** | En `parse_action_call`, reemplazar `.collect::<Result<_,_>>().ok()?` por propagación del `Result` para que el error de parseo de cada argumento se registre como diagnóstico y no se descarte silenciosamente. |
| P3 | **Corrección de implementación** | Endurecer `looks_like_assignment` para rechazar nombres que empiezan por dígito y evitar confusión con `==`; validar el delimitador antes de considerar una línea como asignación. |
| P4 | **Corrección de implementación** | Revisar `operator_text` y la implementación de `Display` para `Token` para que los operadores no produzcan texto con comillas inesperado; asegurar que solo se use para `or`/`and` o se corrija su contrato. |
| P5 | **Corrección de implementación** | Evitar ejecutar `validate_header_value` cuando el parseo previo a `i64` ya haya fallado, para no generar diagnósticos duplicados por el mismo valor de header. |
| P6 | **Corrección de implementación** | Mantener fail-fast en el parser (primer error detiene) como contrato actual; no se añade recuperación de errores en esta etapa. |
| P7 | **Pruebas** | Agregar pruebas en `compile_tests.rs` para header con valores negativos, `lifespan` cero, claves mayúsculas, `if` con `goto` dentro de texto, acción con args vacíos, múltiples errores en una línea y orden de diagnósticos. |

---

## 3. Intérprete (`src/interp.rs`)

### Problemas detectados

| # | Problema | Evidencia |
|---|----------|-----------|
| I1 | **`unwrap_or(0)` silencioso en reinicio a `start`**: si el programa no tiene label `start`, se reinicia en 0 ocultando un programa mal formado. | `interp.rs:194` |
| I2 | **Límite de líneas off-by-one**: `lines_executed` se incrementa antes de comprobar `LINE_LIMIT`; el 101º acceso se fetch pero no se ejecuta, y `pc` queda en la línea del corte. | `interp.rs:167-178` |
| I3 | **`sleep_turns` gestionado externamente**: el intérprete solo decrementa; nunca lo escribe a partir de la acción `wait`. Rompe encapsulación y obliga a hacks en tests. | `interp.rs:157-163`, `tests/interp_tests.rs:191, 209` |
| I4 | **Posible `pc` fuera de rango**: `execute_line` puede asignar `pc` desde `resolve_label`; si el programa se muta externamente, `pc` puede quedar ≥ `len` y provocar panic en la siguiente iteración. | `interp.rs:79-202` |
| I5 | **Falta de pruebas**: `IfGoto` verdadero/falso y con error, `Goto` a label desconocida, error de evaluación de argumentos de acción, `say` múltiple, overflow aritmético, `rem` por cero, negación de `i64::MIN`. | `tests/interp_tests.rs:78-253` |
| I6 | **Sin emisión de eventos internos**: el comentario menciona `RuntimeError` event pero el intérprete solo devuelve `Result<_, (Diagnostic, usize)>`. | `interp.rs:149` |

### Resolución de problemas

| ID | Tipo de resolución | Qué se hace exactamente |
|----|-------------------|-------------------------|
| I1 | **Patrón de diseño: Result / Option idiomático** | Reemplazar `program.resolve_label("start").unwrap_or(0)` por propagación de error (`Err(Diagnostic)`) cuando falta el label `start`; no se permite fallback silencioso a 0. |
| I2 | **Corrección de implementación** | Corregir el off-by-one en `LINE_LIMIT`: comprobar el límite antes de incrementar `lines_executed` o mover el incremento después de la ejecución efectiva, dejando `pc` en una línea válida. |
| I3 | **Patrón de diseño: State** | Modelar el estado de la criatura como `CreatureState { Awake, Sleeping(u32), Dead }`; el intérprete delega el comportamiento al estado en lugar de gestionar `sleep_turns` desde fuera, eliminando hacks externos. |
| I4 | **Corrección de implementación** | Validar que `pc` esté dentro de los límites de `program.lines.len()` antes de usarlo tras un `goto`/`resolve_label`, o asegurar el invariante de que `resolve_label` solo devuelve índices válidos. |
| I5 | **Pruebas** | Agregar pruebas en `interp_tests.rs` para `IfGoto` verdadero/falso y con error, `Goto` a label desconocida, error de evaluación de argumentos de acción, `say` múltiple en un turno, overflow aritmético, `rem` por cero y negación de `i64::MIN`. |
| I6 | **Patrón de diseño: Observer / Event Bus** | Formalizar la emisión de eventos internos (`RuntimeError`, y los que correspondan) desde el intérprete hacia un bus de eventos, para que consumidores externos (CLI, tests) reaccionen sin que el núcleo use `println!`. |

---

## 4. Mundo y simulación (`src/world.rs`, `src/sim.rs`, `src/species.rs`, `src/resources.rs`, `src/perception.rs`, `src/actions.rs`)

### Problemas detectados

| # | Problema | Evidencia |
|---|----------|-----------|
| W1 | **`expect` en `World::draw`**: `expect("grid references a missing creature")` panea si la rejilla contiene un id huérfano. | `world.rs:580` |
| W2 | **Desacoplo de catálogos Mundo vs Simulación**: `World` clona `terrains`/`objects`; `Simulation` mantiene los suyos propios. `strike_cell`/`damage_cell_content` mutan los del `World`, pero `end_of_tick_terrain_and_objects` regenera sobre los de `Simulation`, desincronizando reservas. | `world.rs:70`, `sim.rs:19, 54-560` |
| W3 | **Cast inseguro a `usize` en `Move::apply`**: convierte `nx as usize`/`ny as usize` sin comprobar signo; un negativo envuelve a gran `usize`. | `actions.rs:346-441` |
| W4 | **Overflow aritmético sin `checked_*` en `end_of_tick_creatures`**: `health - 1` y `age + 1` panican en debug con `i64::MIN`. | `sim.rs:550-515` |
| W5 | **Búsqueda lineal repetida y `dead_ids.contains` O(n²)**: `creature`, `creature_mut`, `creature_position` recorren `self.creatures` cada acceso; `dead_ids.contains` dentro de bucle es O(n²). | `world.rs:203-496`, `sim.rs:90-169` |
| W6 | **Falta de pruebas**: `damage_cell_content` overflow y no-op fuera de mapa; `move_creature_to` con ids inexistentes o fuera de rango; `apply_action_result` con id desconocido; `set_creature_health`/`set_variable` con id desconocido; overflow de `i64::MIN` en fin de tick. | `tests/world_tests.rs`, `tests/sim_tests.rs` |

### Resolución de problemas

| ID | Tipo de resolución | Qué se hace exactamente |
|----|-------------------|-------------------------|
| W1 | **Patrón de diseño: Result / Option idiomático** | Cambiar `World::draw` a devolver `Result<String, Diagnostic>` en lugar de usar `expect`; eliminar el panic ante id huérfano en la rejilla. |
| W2 | **Patrón de diseño: Puertos y adaptadores** | Unificar el propietario de `terrains` y `objects` para que `World` y `Simulation` no mantengan copias desincronizadas; `Simulation` es dueño y `World` accede a través de referencias o clones consistentes, evitando que `end_of_tick_terrain_and_objects` regenere sobre una copia diferente. |
| W3 | **Corrección de implementación** | Validar el signo de `nx` y `ny` antes del cast a `usize` en `Move::apply`; rechazar coordenadas negativas como error en lugar de permitir wrap-around. |
| W4 | **Corrección de implementación** | Usar `checked_add` / `checked_sub` en `end_of_tick_creatures` para `health - 1` y `age + 1`; saturar o emitir evento en lugar de permitir panic en debug con `i64::MIN`. |
| W5 | **Patrón de diseño: ECS ligero** | Reemplazar `Vec<Creature>` con búsqueda lineal por almacenes indexados por `CreatureId` (O(1)); eliminar `dead_ids.contains` O(n²) dentro de bucles. La rejilla sigue guardando `CellContent::Creature(CreatureId)` (identificadores estables, no referencias). |
| W6 | **Pruebas** | Agregar pruebas en `world_tests.rs` y `sim_tests.rs` para `damage_cell_content` overflow y no-op fuera de mapa, `move_creature_to` con ids inexistentes o fuera de rango, `apply_action_result` con id desconocido, `set_creature_health`/`set_variable` con id desconocido, y overflow de `i64::MIN` en fin de tick. |

---

## 5. Arquitectura y núcleo (`src/lib.rs`, `src/program.rs`, `src/value.rs`, `src/diagnostic.rs`, `src/event.rs`)

### Problemas detectados

| # | Problema | Evidencia |
|---|----------|-----------|
| A1 | **`unwrap_or(0)` silencioso en `interp` al faltar label `start`**: no es un panic, pero oculta programas mal formed. | `interp.rs:194` (visto desde núcleo) |
| A2 | **`unwrap` en parser (`expr.rs`)**: `self.advance().unwrap()` asume invariante de `peek`. | `expr.rs:267-281` |
| A3 | **`expect` en `world::draw`**: panico si la rejilla referencia id inexistente. | `world.rs:580` |
| A4 | **`Program::labels` es `HashMap<String,usize>`**: resolución determinista vía `get`, pero iteración no determinista (no afecta lógica actual, sí logs/serialización futura). | `program.rs:70-88` |
| A5 | **Cobertura de pruebas del núcleo débil**: faltan pruebas para `to_bool_int`, comparación de orden con ints, `Display` de cada variante de `Event` (`Died`, `RuntimeError`, `Damaged`, `Roared`), y no existe `program_tests.rs`. | `tests/value_tests.rs`, `tests/event_tests.rs` |

### Resolución de problemas

| ID | Tipo de resolución | Qué se hace exactamente |
|----|-------------------|-------------------------|
| A1 | **Patrón de diseño: Result / Option idiomático** | Reemplazar `unwrap_or(0)` por propagación de error cuando falta `start`; consiste en la misma corrección que I1, vista desde el núcleo. |
| A2 | **Patrón de diseño: Result / Option idiomático** | Reemplazar `unwrap` en `Parser::expect`/`advance`; consiste en la misma corrección que P1, vista desde el núcleo. |
| A3 | **Patrón de diseño: Result / Option idiomático** | Reemplazar `expect` en `World::draw`; consiste en la misma corrección que W1, vista desde el núcleo. |
| A4 | **Corrección de implementación / Mejora de arquitectura** | Evaluar si se sustituye `HashMap<String,usize>` por `BTreeMap` o `IndexMap` para obtener orden determinista de iteración; no es un error bloqueante, solo una mejora para logs y serialización futura. |
| A5 | **Pruebas** | Agregar pruebas en `value_tests.rs` (`to_bool_int`, comparación de orden con ints), `event_tests.rs` (`Display` de cada variante de `Event`), y crear `program_tests.rs` para `Program::resolve_label` y `Line::clone`. |

---

## 6. CLI (`crates/instinct-cli/src/main.rs`, `tests/cli_*.rs`)

### Problemas detectados

| # | Problema | Evidencia |
|---|----------|-----------|
| C1 | **Flags desconocidos ignorados silenciosamente** en `parse_run`. | `main.rs:252-254` |
| C2 | **Validación numérica insuficiente**: `--ticks` se parsea como `i64` sin comprobar ≥ 0; `ticks as u64` hace wrap-around si es negativo. | `main.rs:265, 167-172` |
| C3 | **`--root` sin validación de existencia**: default `"examples"` sin comprobar que el directorio exista. | `main.rs:258` |
| C4 | **Falta de pruebas**: `run` sin `--map`, `--root` inexistente, `--place` mal formado, especie desconocida, colocación fuera de límites, `--every <= 0`, `--ticks` negativo, `check` con número incorrecto de argumentos. | `tests/cli_check.rs`, `tests/cli_run.rs`, `tests/cli_usage.rs` |
| C5 | **Mezcla de streams**: eventos de simulación y diagnósticos de carga usan `stdout`/`stderr` de forma no consistente. | `main.rs:59-175`, `281-310` |

### Resolución de problemas

| ID | Tipo de resolución | Qué se hace exactamente |
|----|-------------------|-------------------------|
| C1 | **Patrón de diseño: Command para subcomandos de CLI** | Reemplazar el `match` manual y el parsing de `std::env::args()` por objetos comando (`Run`, `Check`, `Help`) que rechacen explícitamente flags desconocidos en lugar de ignorarlos silenciosamente. |
| C2 | **Patrón de diseño: Builder para argumentos de ejecución (`RunArgs`)** | Validar `--ticks >= 0` y `--every > 0` antes de construir `RunArgs`; rechazar valores negativos para evitar wrap-around al convertir a `u64`. |
| C3 | **Patrón de diseño: Builder para argumentos de ejecución (`RunArgs`)** | Validar la existencia del directorio `--root` antes de usarlo; no aceptar valores por defecto silenciosos sin verificar. |
| C4 | **Pruebas** | Agregar pruebas en `cli_*.rs` para `run` sin `--map`, `--root` inexistente, `--place` mal formado, especie desconocida, colocación fuera de límites, `--every <= 0`, `--ticks` negativo y `check` con número incorrecto de argumentos. |
| C5 | **Patrón de diseño: Separación de streams (stdout vs stderr)** | Enviar salida de simulación (mapa, eventos) a `stdout` y diagnósticos/errores a `stderr`, separando claramente los flujos. |

---

## Matriz resumen: problema → tipo de resolución → patrón o acción requerida

| Parte | ID | Tipo de resolución | Patrón / Acción requerida |
|-------|----|-------------------|---------------------------|
| Lexer | L1 | Patrón de diseño | Error Chain / Diagnostic unificado |
| Lexer | L2 | Corrección de implementación | Normalizar salto de línea `\r\n` y definir contrato de `lex` |
| Lexer | L3 | Feature nuevo | Soportar escapes `\"` y `\\` en textos |
| Lexer | L4 | Pruebas | Ampliar `lexer_tests.rs` |
| Lexer | L5 | Corrección de implementación | Decidir soporte Unicode vs ASCII |
| Lexer | L6 | Corrección de implementación | Mantener fail-fast como contrato actual |
| Parser/AST | P1 | Patrón de diseño | Result / Option idiomático |
| Parser/AST | P2 | Corrección de implementación | Propagar `Result` de args en `parse_action_call` |
| Parser/AST | P3 | Corrección de implementación | Endurecer `looks_like_assignment` |
| Parser/AST | P4 | Corrección de implementación | Corregir `operator_text` y `Display` de `Token` |
| Parser/AST | P5 | Corrección de implementación | Evitar validación duplicada de header |
| Parser/AST | P6 | Corrección de implementación | Mantener fail-fast en parser |
| Parser/AST | P7 | Pruebas | Ampliar `compile_tests.rs` |
| Intérprete | I1 | Patrón de diseño | Result / Option idiomático |
| Intérprete | I2 | Corrección de implementación | Corregir off-by-one en `LINE_LIMIT` |
| Intérprete | I3 | Patrón de diseño | State (`CreatureState`) |
| Intérprete | I4 | Corrección de implementación | Validar rango de `pc` tras `goto`/`resolve_label` |
| Intérprete | I5 | Pruebas | Ampliar `interp_tests.rs` |
| Intérprete | I6 | Patrón de diseño | Observer / Event Bus |
| Mundo/sim. | W1 | Patrón de diseño | Result / Option idiomático |
| Mundo/sim. | W2 | Patrón de diseño | Puertos y adaptadores |
| Mundo/sim. | W3 | Corrección de implementación | Validar signo antes de cast a `usize` en `Move` |
| Mundo/sim. | W4 | Corrección de implementación | Usar `checked_add/sub` en fin de tick |
| Mundo/sim. | W5 | Patrón de diseño | ECS ligero (almacenes indexados por `CreatureId`) |
| Mundo/sim. | W6 | Pruebas | Ampliar `world_tests.rs` y `sim_tests.rs` |
| Núcleo | A1 | Patrón de diseño | Result / Option idiomático |
| Núcleo | A2 | Patrón de diseño | Result / Option idiomático |
| Núcleo | A3 | Patrón de diseño | Result / Option idiomático |
| Núcleo | A4 | Corrección / Mejora | Evaluar `BTreeMap`/`IndexMap` para `labels` |
| Núcleo | A5 | Pruebas | Ampliar pruebas de núcleo y crear `program_tests.rs` |
| CLI | C1 | Patrón de diseño | Command para subcomandos de CLI |
| CLI | C2 | Patrón de diseño | Builder para `RunArgs` (validar `--ticks >= 0`) |
| CLI | C3 | Patrón de diseño | Builder para `RunArgs` (validar existencia de `--root`) |
| CLI | C4 | Pruebas | Ampliar `cli_*.rs` |
| CLI | C5 | Patrón de diseño | Separación de streams (stdout vs stderr) |

### Próximos pasos sugeridos para el plan

1. Ejecutar `cargo test --workspace` para tener una línea de base verde antes de cambios.
2. Ejecutar la **Etapa A** (correcciones de implementación, features y tests): todos los problemas marcados como "Corrección de implementación", "Feature nuevo" y "Pruebas".
3. Ejecutar la **Etapa B** (patrones de diseño): aplicar los patrones listados en la matriz sobre el código ya estable de la Etapa A.
4. Tras cada cambio, correr `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check`.
5. Invocar al subagente `reviewer` para auditar el diff antes de dar por cerrada cada parte.
