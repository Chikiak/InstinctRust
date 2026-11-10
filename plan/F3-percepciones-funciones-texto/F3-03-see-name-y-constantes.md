---
id: F3-03
title: see y name con las cinco constantes
phase: F3
status: done
depends_on: [F3-02]
size: L
---

# F3-03 · `see` y `name` con las cinco constantes

## Objetivo

Implementar las dos funciones reservadas del lenguaje, `see` y `name`, cada una con dos argumentos
enteros de coordenada, con sus nombres y aridades resueltos al compilar contra un catálogo por
nombre, y las cinco constantes (`NONE`, `GROUND`, `OBJECT`, `ALLY`, `ENEMY`), de modo que una
criatura pueda preguntar por cualquier casilla de su radio y comparar el resultado con una
constante.

## Alcance

**Haz**
- Módulo nuevo `crates/instinct-core/src/functions.rs` con el trait `Function` (`name`, `arity`,
      `call`, `clone_box`) y su catálogo por nombre (`register`, `resolve` con validación de
      aridad, `contains`, catálogo por defecto), siguiendo el patrón de `Action`/`Catalog`.
- Los tipos `See` y `Name` con aridad 2: evalúan sus argumentos y consultan la vista de la
      criatura; argumentos de texto son error de ejecución.
- Nodo `Call` en el AST de `expr.rs`, que guarda la función ya resuelta y sus argumentos como
      nodos; `call` recibe la vista y los valores evaluados.
- `EvalContext` gane `view()`; `CreatureContext` guarde la instantánea y la devuelva; los contextos
      de prueba sin vista hacen que la llamada falle con error de ejecución.
- `expr::parse` reciba además el catálogo de funciones: nombre desconocado y aridad distinta son
      error de parseo con archivo y línea. `parse_primary` acepta una llamada con dos argumentos
      cualesquiera.
- `Catalog` gane el catálogo de funciones y la tabla de constantes con `resolve_function`,
      `is_function_name`, `resolve_constant`, `is_constant_name`, y un acceso público al catálogo de
      funciones para el parser; `Catalog::default()` registra `see`, `name` y las cinco constantes.
- `CreatureView` gane los dos métodos de consulta de casilla (`see` y `name`);
      `CreatureSnapshot` las implementa delegando en consultas nuevas de `World`.
- `compile` rechaza asignar a nombre de función y a constante; `World::set_variable` hace lo mismo.
- Pruebas de `see`/`nombre` contra el mundo real, errores de compilación de función desconocida y
      aridad, valores de las constantes, y error de ejecución por argumento de texto.
- `cargo test -p instinct-core`, suite completa, clippy y fmt en verde.

**No hagas**
- No implementes percepciones nuevas (F3-01 y F3-02 las cubren todas).
- No amplíes el lenguaje con más funciones (`distance`, `reserve`, `health_at`): no son de la base.
- No conviertas `see`/`name` en percepciones con argumentos ni en acciones.
- No cambies la semántica de `and`/`or`/`not`, de `==`/`!=` ni de los errores de tipo de `value.rs`.
- No toques el evento `Roared` ni la rotación de rugidos.

### Archivos previstos
- `crates/instinct-core/src/functions.rs` — nuevo — trait `Function`, catálogo, `See`, `Name`.
- `crates/instinct-core/src/lib.rs` — modificar — declarar el módulo nuevo.
- `crates/instinct-core/src/expr.rs` — modificar — nodo `Call`, `view()` en `EvalContext`, catálogo
      de funciones en `parse`, aceptar llamadas en `parse_primary`.
- `crates/instinct-core/src/compile.rs` — modificar — pasar el catálogo de funciones a `expr::parse`
      en sus tres sitios y rechazar asignación a función y constante.
- `crates/instinct-core/src/perception.rs` — modificar — los dos métodos de consulta de casilla en
      `CreatureView`.
- `crates/instinct-core/src/world.rs` — modificar — implementación de `see`/`name` en
      `CreatureSnapshot`, consultas nuevas de `World`, `set_variable` rechaza función y constante.
- `crates/instinct-core/src/actions.rs` — modificar — campos de funciones y constantes en `Catalog`,
      registro por defecto, accesores.
- `crates/instinct-core/tests/functions_tests.rs` — nuevo — `see`, `name` y constantes contra el
      mundo real.
- `crates/instinct-core/tests/expr_tests.rs` — modificar — llamadas, función desconocida, aridad.
- `crates/instinct-core/tests/compile_tests.rs` — modificar — asignación a función y constante;
      ajuste de la prueba que hoy afirma `unknown function` para una llamada cualquiera.
- `crates/instinct-core/tests/perception_tests.rs` — modificar — doble `EmptyView` ampliado y
      llamadas a `expr::parse` con el catálogo nuevo.
- `crates/instinct-core/tests/sim_tests.rs` — modificar — error de ejecución por texto y uso de
      `see`/`name` en una criatura.

## Criterios de aceptación

- [ ] **AC1** `see` con dos argumentos devuelve `GROUND` con solo terreno, `OBJECT` con objeto, `ALLY` con criatura de la propia facción (incluida la casilla propia) y `ENEMY` con criatura de otra facción. Verif: `test_see_returns_constant_per_content` | `cargo test -p instinct-core --test functions_tests`.
- [ ] **AC2** `see` devuelve `NONE` cuando la distancia de Chebyshev supera `vision`, y esa comprobación va antes que cualquier otra; una casilla fuera del mapa pero dentro del radio devuelve `OBJECT`. Verif: `test_see_outside_radius_and_map` | `cargo test -p instinct-core --test functions_tests`.
- [ ] **AC3** `name` devuelve el nombre de la especie con criatura, el del objeto con objeto y el del terreno en otro caso, y el texto vacío fuera del mapa o fuera del radio. Verif: `test_name_returns_element_name` | `cargo test -p instinct-core --test functions_tests`.
- [ ] **AC4** Las cinco constantes resuelven a -1, 0, 1, 2 y 3, y el resultado de `see` compara igual que una constante: preguntar por la casilla de la derecha y comparar con `GROUND` da 1 cuando solo hay terreno. Verif: `test_constants_compare_with_see` | `cargo test -p instinct-core --test functions_tests`.
- [ ] **AC5** Un nombre de función que no está en el catálogo y una aridad distinta de dos (incluida la llamada sin argumentos) son errores de compilación con archivo y línea, y el mensaje nombra la función. Verif: `test_compile_rejects_unknown_function`, `test_compile_rejects_wrong_arity` | `cargo test -p instinct-core --test compile_tests`.
- [ ] **AC6** La resolución es por catálogo y sin cadena de `if` por nombre: el nodo `Call` guarda la función ya resuelta y sus argumentos pueden ser expresiones arbitrarias, incluidas percepciones y sumas de coordenadas. Verif: `test_call_arguments_are_expressions` | `cargo test -p instinct-core --test expr_tests`.
- [ ] **AC7** Asignar a un nombre de función o de constante es error de compilación, y `World::set_variable` también lo rechaza por API. Verif: `test_compile_rejects_assignment_to_function_and_constant` | `cargo test -p instinct-core --test compile_tests`.
- [ ] **AC8** Un argumento que evalúa a texto en `see` o `name` es error de ejecución que mata a la criatura con su línea; una criatura que usa ambas funciones con argumentos enteros corre sin errores. Verif: `test_see_with_text_argument_kills_creature`, `test_creature_uses_see_and_name` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC9** La nueva firma de `expr::parse` y de `EvalContext` está propagada: los tres sitios de `compile.rs`, `tests/perception_tests.rs` y los dobles de prueba compilan; la prueba `test_function_in_expression_reports_on_its_line` deja de usar `see` (ahora válido) y afirma el mismo contrato con un nombre de función desconocido, sin suavizar la aserción. Verif: `cargo test -p instinct-core --test expr_tests`.
- [ ] **AC10** Suite completa sin regresiones y puertas de calidad en verde. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `expr.rs` con el parser de descenso recursivo y el rechazo fijo de llamadas; `Catalog`
y `Catalog::default()`; `parse_assignment` rechazando palabras reservadas, percepciones y acciones;
`CreatureView` con los accesores de F3-01 y `nearby` de F3-02; `CreatureContext` con `resolve`
en `world.rs`; `value.rs` con la semántica de texto; `actions.rs` con el patrón de acción resuelta
al compilar y guardada en `Line::Action`.

**Contratos que debes respetar**: los de `PHASE.md#contratos-compartidos` — funciones resueltas al
comprobadas al compilar y guardadas resueltas, constantes en el catálogo único, orden de
resolución percepción →
constante → variable, cambios de firma propagados en el mismo issue con las pruebas ajustadas.

**Ojo**: `see` comprueba el radio antes que nada, así que una casilla dentro del radio pero fuera
del mapa es `OBJECT` (el borde es un muro) mientras que una casilla lejana es `NONE` aunque exista.
La propia casilla de quien pregunta es `ALLY` y `name` devuelve su propia especie. Las constantes
son nombres como cualquier otro en el léxico: no hace falta tocarlo, basta resolverlos en la vista.
Mantén el mensaje `unknown function 'x'` para el nombre desconocido, porque ya existen pruebas que
lo afirman.

### Leer primero
- `crates/instinct-core/src/actions.rs` — `Action`, `Catalog`, `Catalog::default()` y la variante
      `Action` de `Line` en `crates/instinct-core/src/program.rs`: el patrón de resolución al
      compilar que `Function` imita.
- `crates/instinct-core/src/expr.rs` — `parse_primary`, `Parser`, `operator_text` y las pruebas de
      sintaxis: dónde nacen las llamadas y sus errores.
- `crates/instinct-core/src/compile.rs` — los tres sitios de `expr::parse` y `parse_assignment`.
- `crates/instinct-core/src/world.rs` — `CreatureSnapshot`, `CreatureContext` y las consultas de
      celda (`content`, `terrains`, `objects`) que `see`/`name` necesitan.
- `crates/instinct-core/src/value.rs` — errores de tipo y `Value::Text`, para el mensaje de
      argumento de texto.

## Especificación

| Función | Aridad | Argumentos | Devuelve |
|---|---|---|---|
| `see` | 2 | dos enteros, coordenadas de casilla | entero: -1, 0, 1, 2 o 3 |
| `name` | 2 | dos enteros, coordenadas de casilla | texto con el nombre del elemento |

Orden de `see`: si la distancia de Chebyshev desde la casilla de la criatura supera `vision` →
`NONE`; si la casilla está fuera del mapa → `OBJECT`; si contiene criatura → `ALLY` con facción
igual y `ENEMY` en otro caso; si contiene objeto → `OBJECT`; en otro caso → `GROUND`. La casilla
propia contiene criatura, así que devuelve `ALLY`.

`name`: fuera del mapa o fuera del radio → texto vacío; criatura → nombre de la especie; objeto →
nombre del objeto; en otro caso → nombre del terreno.

Constantes: `NONE` -1, `GROUND` 0, `OBJECT` 1, `ALLY` 2, `ENEMY` 3. Resolución de un nombre en una
expresión: primero percepción, después constante, después variable. Un argumento que evalúa a texto
en `see`/`name` es error de ejecución que nombra el argumento, igual que en las acciones numéricas.

### Pasos sugeridos (opcional)
1. Escribe primero las pruebas de `see`/`name` y de las constantes contra el mundo real, y las de
   parseo de llamadas; deben fallar.
2. Crea `functions.rs` con el trait, el catálogo y los dos tipos.
3. Añade `Call` y `view()`, y enseña al parser a resolver llamadas contra el catálogo.
4. Propaga la nueva firma de `expr::parse` y añade constantes y funciones al `Catalog`.
5. Implementa `see`/`name` en la instantánea con las consultas de `World`, y cierra con los errores
   de asignación y de ejecución.

## Tests

Nivel: unitario e integración. `crates/instinct-core/tests/functions_tests.rs` (nuevo) con el mundo
real y el catálogo real: cada valor de `see` según contenido, `NONE` fuera de radio, `OBJECT` fuera
del mapa dentro del radio, `name` para cada tipo de elemento y texto vacío, valores de las cinco
constantes. En `expr_tests.rs`: llamadas con expresiones como argumentos, función desconocida y
aridad incorrecta. En `compile_tests.rs`: asignación a función y constante. En `sim_tests.rs`:
argumento de texto como error de ejecución y una criatura que usa `see`/`name` sin errores. Nada de
contextos falsos salvo `EmptyView`.

## Verificación
- `cargo test -p instinct-core --test functions_tests` → `see`, `name` y constantes pasan.
- `cargo test -p instinct-core --test expr_tests` → llamadas, función desconocida y aridad pasan.
- `cargo test -p instinct-core --test compile_tests` → asignación rechazada y firma nueva pasan.
- `cargo test -p instinct-core --test sim_tests` → error de ejecución y uso real pasan.
- `cargo test --workspace` → sin regresiones de F0, F1, F2, F3-01 y F3-02.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin
stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si `view()` no encaja en `EvalContext` sin romper los contextos de prueba existentes, para y
repórtalo antes de cambiar el trait de otra forma. Decisiones que NO puedes tomar tú: resolver
nombres de función en tiempo de ejecución, añadir más funciones, cambiar el orden de resolución
percepción → constante → variable, o suavizar una prueba existente para que compile.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Commit: 400dd61 (rama feat/f3-03-see-name-constantes, sin push). `plan/.../NOCHE_F3.md` dejado sin trackear/commit (rascacho fuera de alcance).

Evidencia AC por AC:
- AC1 PASS: `test_see_returns_constant_per_content` (functions_tests) — GROUND/OBJECT/ALLY/ENEMY por contenido; casilla propia ALLY. `cargo test -p instinct-core --test functions_tests` 5/5.
- AC2 PASS: `test_see_outside_radius_and_map` — NONE (-1) fuera de radio (radio antes que todo), OBJECT fuera del mapa dentro del radio. Mismo comando.
- AC3 PASS: `test_name_returns_element_name` — especie/objeto/terreno y texto vacío fuera de radio/mapa. Mismo comando.
- AC4 PASS: `test_constants_compare_with_see` — NONE -1, GROUND 0, OBJECT 1, ALLY 2, ENEMY 3; `see(4, 2) == GROUND` → 1. Mismo comando.
- AC5 PASS: `test_compile_rejects_unknown_function`, `test_compile_rejects_wrong_arity` (1, 0 args) con archivo/línea y mensaje que nombra la función. `cargo test -p instinct-core --test compile_tests` 37/37.
- AC6 PASS: `test_call_arguments_are_expressions` (expr_tests) — `Call` guarda función resuelta; argumentos expresiones arbitrarias (`see(1 + 0, 2)`); aridad 0/1/3 errores de parseo. `cargo test -p instinct-core --test expr_tests` 10/10.
- AC7 PASS: `test_compile_rejects_assignment_to_function_and_constant` — see/name/NONE/GROUND/OBJECT/ALLY/ENEMY rechazados en compile y en `World::set_variable` (API). Mismo comando compile_tests.
- AC8 PASS: `test_see_with_text_argument_kills_creature` (RuntimeError línea 7, causa error) y `test_creature_uses_see_and_name` (sin errores, criatura viva). `cargo test -p instinct-core --test sim_tests` 15/15.
- AC9 PASS: `test_function_in_expression_reports_on_its_line` usa `bogus` (ahora `see` es válido) con línea; `perception_tests.rs::eval` pasa `Catalog::default().functions()` a `expr::parse`; `EmptyView` implementa `see`/`name`.
- AC10 PASS: `cargo test --workspace` todo verde; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all --check` exit 0.

Firmas cambiadas (autorizadas en PHASE.md#cambios-de-firma, propagadas en el mismo cambio):
- `expr::parse(text, line, file, functions: &FunctionCatalog)` — 3 sitios en compile.rs (if-goto, assign, args de acción) + tests/perception_tests.rs.
- `EvalContext`: gana `view() -> Option<&dyn CreatureView>` (default None). `CreatureContext` guarda `CreatureSnapshot` y lo devuelve; `FakeContext::view()` → None.
- `CreatureView`: +`see(x, y) -> i64`, +`name(x, y) -> &str`; propagado a `CreatureSnapshot` (world.rs), `FixedView` (pruebas de src/perception.rs), `EmptyView` (tests/perception_tests.rs).
- `Catalog`: +`functions: FunctionCatalog`, `resolve_function`, `is_function_name`, `resolve_constant`, `is_constant_name`, `functions()`; `Catalog::default()` registra `see`, `name` y las 5 constantes (doble registro con `FunctionCatalog::default()`).
- `World::set_variable` rechaza también funciones y constantes.

Nuevos archivos: `crates/instinct-core/src/functions.rs` (trait `Function`, `FunctionCatalog`, `See`, `Name`, `arg_as_int`) y `crates/instinct-core/tests/functions_tests.rs`. Nodo `expr::Call` (función resuelta + argumentos). `compile.rs` gana `split_action_args` (comas de nivel superior, consciente de paréntesis y textos) para permitir llamadas anidadas como argumentos de acción (p. ej. `say(see(1, 0))`).

Hechos para F3-04: `see`/`name` y constantes resuelven por catálogo sin cadenas de `if` por nombre; orden de resolución de nombres: percepción → constante → variable.
