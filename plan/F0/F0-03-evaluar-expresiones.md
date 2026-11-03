---
id: F0-03
title: Evaluar expresiones sobre enteros y texto
phase: F0
status: todo
depends_on: [F0-01]
size: M
---

# F0-03 · Evaluar expresiones sobre enteros y texto

## Objetivo
Que el núcleo pueda leer el texto de una expresión de Instinct y evaluarla con las reglas de tipos, precedencia y errores del enunciado. Es el corazón del intérprete: fija el árbol de expresiones como jerarquía de nodos que se evalúan solos.

## Alcance
**Haz**
- `Value` con sus operaciones y la semántica de la fase (división hacia menos infinito, resto con signo del divisor, aritmética con detección de desbordamiento, verdad, tipos).
- Lexer de una línea de texto: nombres, enteros, textos entre comillas, operadores, paréntesis, comas, comentario final, con el número de línea.
- Parser de expresiones con la precedencia de la fase y árbol de nodos que implementan un trait común de evaluación.
- Un contexto de evaluación mínimo (trait) que resuelve nombres a valores y permite probar con valores falsos.

**No hagas**
- Percepciones reales, acciones ni parseo de líneas completas del `.ins` (F0-05 y F0-06).
- Llamadas a funciones: una llamada en una expresión es error de compilación por función desconocida.
- Un enum cerrado con un `match` central de evaluación: cada nodo se evalúa a sí mismo.

### Archivos previstos
- `crates/instinct-core/src/value.rs` — nuevo — `Value` y sus operaciones.
- `crates/instinct-core/src/lexer.rs` — nuevo — tokens con línea.
- `crates/instinct-core/src/expr.rs` — nuevo — trait de expresión, nodos y parser.
- `crates/instinct-core/src/lib.rs` — modificar — exportar los módulos.

## Criterios de aceptación
- [ ] **AC1** La precedencia y los paréntesis dan estos resultados: "2 + 3 * 4" es 14, "(2 + 3) * 4" es 20, "-2 * 3" es -6, "10 - 4 - 3" es 3, "not 0 == 5" es 0 (el `not` ata más fuerte que `==`), "1 or 0 and 0" es 1. Verif: `test_precedence_table`
- [ ] **AC2** División y resto siguen el enunciado: 7 entre 2 es 3, -7 entre 2 es -4, 7 entre -2 es -4, -7 resto 3 es 2, 7 resto -3 es -2; dividir o resto por cero y el desbordamiento de una suma, resta, producto o menos unario son errores de ejecución cuyo mensaje nombra la causa. Verif: `test_floor_division_and_modulo`, `test_division_by_zero_and_overflow`
- [ ] **AC3** Comparaciones y lógica devuelven 1 o 0; el 0 y el texto vacío son falsos; `and` y `or` no evalúan el lado derecho si el izquierdo ya decide (se prueba con un lado derecho que dividiría por cero). Verif: `test_logic_truthiness_and_short_circuit`
- [ ] **AC4** Con textos solo valen `==` y `!=`; sumar o comparar con `<` textos, comparar un entero con un texto y negar un texto son errores de ejecución; `and`, `or` y `not` aceptan cualquier tipo. Verif: `test_text_operations_and_type_errors`
- [ ] **AC5** Un `#` dentro de un texto no inicia comentario y un comentario final se ignora. Verif: `test_hash_inside_string_is_not_a_comment`
- [ ] **AC6** Los errores de sintaxis (paréntesis sin cerrar, operando que falta, texto sin cerrar, carácter desconocido, entero fuera de rango, llamada a función) devuelven un `Diagnostic` con la línea indicada. Verif: `test_expression_syntax_errors_report_line`
- [ ] **AC7** Un nombre se evalúa a través del contexto; si el contexto no lo conoce, el resultado es un error de ejecución que nombra la variable. Verif: `test_names_resolve_through_context`
- [ ] **AC8** Suite completa, clippy y fmt en verde. Verif: los comandos de Verificación

## Contexto
**Ya existe** (F0-01): `Diagnostic` en `crates/instinct-core/src/diagnostic.rs` y la estructura de módulos de `crates/instinct-core/src/lib.rs`.
**Contratos que debes respetar**: semántica de `Value` y precedencia en `../PHASE.md#contratos-compartidos`; A6 a A9 en `docs/ambiguedades.md`.
**Ojo**: el menos de un literal como -7 se trata como el operador unario sobre 7. La división de Rust trunca hacia cero y su resto toma el signo del dividendo: hay que ajustar ambos para igualar al enunciado. El cortocircuito importa también porque leer `random` más adelante consume el generador.
**Rust que se practica**: objetos de trait (`Box<dyn ...>`), recursión en el parser, `match` sobre tokens, `checked_*` en aritmética, enums con datos.

### Leer primero
- `crates/instinct-core/src/diagnostic.rs` — cómo se reporta un error con línea.
- `crates/instinct-core/src/lib.rs` — cómo se exportan los módulos.
- `docs/enunciado.md` — sección 2.5 completa.
- `docs/ambiguedades.md` — A6 a A9.

## Especificación
- Un error de ejecución es un mensaje sin línea (la pone quien ejecuta la línea más adelante); un error de compilación es un `Diagnostic`.
- Tipos de nodo previstos: literal entero, literal de texto, nombre, operación unaria, operación binaria; cada uno implementa el trait de evaluación. Añadir un tipo de nodo no debe obligar a editar los existentes.
- El contexto de evaluación solo sabe devolver el valor de un nombre o indicar que no existe.
- La entrada del parser es el texto de una expresión, el número de línea que se le atribuye y el nombre del archivo.

## Tests
Unitarios, mayormente tabulados (texto de la expresión, resultado esperado o error esperado). Usa un contexto falso con un mapa de nombres a valores: es fiel al contrato y no replica la implementación. Escribe primero la tabla de precedencia y la de división y resto.

## Verificación
- `cargo test -p instinct-core expr` y `cargo test -p instinct-core value` → esperado: AC1 a AC7 en verde.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` → esperado: exit 0.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si la jerarquía de nodos choca con el borrow checker de una forma que obliga a otro diseño, para y repórtalo. No cambies la precedencia ni la semántica de la tabla: son decisiones de la fase.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Añade a `docs/ambiguedades.md` cualquier decisión nueva.
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
