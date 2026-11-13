---
phase: F5
title: Errores y carga resiliente
status: todo
master: ../MASTER.md
fuentes: ../../docs/enunciado.md (secciones 2.8 y carga de recursos), ../../docs/PATRONES_DISENO_ARQUITECTURA.md (6.3)
issues: 5 desglosados en archivos propios (F5-01 a F5-05)
---

# F5 · Errores y carga resiliente

## Objetivo y entregable

Cerrar el catálogo completo de errores de la sección 2.8 del enunciado con su mensaje y línea, y
demostrar que la carga sobrevive a los archivos inválidos. Entregable observable: una carpeta de
archivos inválidos (uno por error de 2.8) con el mensaje y línea esperados, verificada por
pruebas automáticas, y una carga que sigue funcionando con los demás recursos.

## Estado de partida (verificado en el repo, no supuesto)

- **F4 cerrada**: bus de eventos/puerto de salida, State, PRNG inyectable, ECS, Flyweight, Memento,
  Visitor AST, capas verificadas. Línea base verde: `cargo test --workspace`, clippy y fmt en verde.
- `crates/instinct-core/src/diagnostic.rs` — `Diagnostic { file, line, message }` único para toda la
  pila; formato `file:line: message`; línea desde 1.
- `crates/instinct-core/src/resources.rs` — la carga es fail-soft por archivo: un recurso inválido
  produce diagnósticos y se descarta, el resto de la carpeta carga; carpeta ausente = vacía (A2).
- `crates/instinct-core/src/resources.rs::load_map` — hoy un carácter desconocido produce
  diagnóstico con carácter y línea, pero **descarta solo la fila y el mapa se carga**. Contradice el
  enunciado (línea 349: "el mapa se rechaza") y el criterio de salida de esta fase. Los tests
  `crates/instinct-core/tests/resources_tests.rs::test_map_unknown_char_reports_line` y
  `crates/instinct-core/tests/world_tests.rs::test_unknown_character_generates_diagnostic`
  afirman el comportamiento incorrecto.
- `crates/instinct-core/src/compile.rs` — acumula diagnósticos por línea y los ordena; cubre línea no
  reconocida, etiqueta duplicada, `goto` a etiqueta inexistente, cabecera incompleta (línea 1, A11),
  valor inválido de cabecera, asignación a percepción/constante/acción/función, acción desconocida y
  aridad (vía `Catalog::resolve`). El bloque de `looks_like_assignment` está duplicado (líneas 310–324
  repiten 231–243) y `parse_action_call` corta en el primer argumento con error.
- `crates/instinct-core/src/expr.rs` — el parser de expresiones resuelve nombre y aridad de función
  en compilación (`resolve_function`); sus errores ya son `ParseError` con archivo y línea.
- `crates/instinct-core/src/interp.rs` — los errores de ejecución se convierten en
  `Diagnostic::new("interp", line, mensaje)`: el archivo es literal `"interp"`, no el `.ins` de
  origen. `crates/instinct-core/src/event.rs::Event::RuntimeError` lleva tick, especie, posición,
  mensaje y línea, pero **no el archivo**.
- `crates/instinct-core/src/actions.rs` — `dx`/`dy` fuera de `{-1, 0, 1}`, texto donde se espera
  número y overflow se reportan nombrando el argumento; el "archivo" del diagnóstico es el nombre de
  la acción.
- `crates/instinct-cli/src/main.rs` — `check` imprime todos los diagnósticos a stderr (exit 1 si
  hay error); `run` imprime diagnósticos de carga a stderr, mapa y eventos a stdout (C5 cerrada).
- Fixtures existentes: `crates/instinct-cli/tests/fixtures/` con `bad_action.ins`, `bad_header.ins`,
  `bad_label.ins`. No existe batería ni carpeta de carga mixta.

## Contratos compartidos

- **Un diagnóstico, un origen**: cada error (compilación, carga, ejecución) se reporta como
  `Diagnostic` con el archivo real donde se detectó y línea desde 1. Los errores de ejecución
  propagan el `.ins` de la criatura, no `"interp"` ni el nombre de la acción.
- **Fail-soft de carga**: un archivo que no carga produce sus diagnósticos y se descarta; los demás
  archivos de la carpeta y las demás carpetas siguen cargando; carpeta ausente = vacía sin
  diagnóstico. Este contrato no cambia en la fase.
- **Un error de ejecución mata solo a su criatura**: el mundo aplica `restore` desde el memento
  (F4-06), emite `RuntimeError` y `Died` (causa `error`), y el tick continúa. No cambia en la fase.
- **Mensajes en inglés, sin cambiar los existentes**: los mensajes actuales se mantienen; la fase
  añade cobertura y pruebas, no reescribe el catálogo. Los mensajes nuevos siguen el estilo de los
  existentes: nombran el elemento ofensor (clave, carácter, etiqueta, argumento).
- **Batería como contrato**: la tabla error → mensaje esperado → línea esperada vive en el `README`
  de la carpeta de archivos inválidos; las pruebas la comprueban, no la duplican.

## Decisiones de la fase

1. **El mapa con carácter desconocido se rechaza entero** (F5-01), con carácter y línea, porque lo
   pide el enunciado y el criterio de salida de F5; el comportamiento anterior (descartar fila) era
   incorrecto. Los dos tests que afirman lo contrario se actualizan en el mismo issue, con
   autorización explícita en el issue.
2. **La batería de archivos inválidos vive en `crates/instinct-cli/tests/fixtures/errors/`** con su
   `README` de mensajes y líneas esperados; `examples/` se mantiene solo con entradas válidas para
   no contaminar las corridas de referencia.
3. **El error de ejecución lleva archivo real** (F5-04): `Species` guarda el origen del `.ins`
   compilado desde archivo y el evento `RuntimeError` gana el campo `file`. Es un cambio declarado
   de la API del núcleo; los sitios de llamada se actualizan en el mismo issue.
4. **El lexer mantiene fail-fast** (contrato L6/decisión de F2). La acumulación (patrón 6.3 Either)
   se aplica donde aporta: recogida de todos los errores de argumentos de una acción y reporte por
   línea en `check`. No se abre el contrato del lexer.
5. **Nombres duplicados de mapa o especie no se validan** en esta fase: el enunciado no lo pide y
   no hay un comportamiento esperado que cubrir. Queda anotado para F6 (banco) si hace falta.
6. **`check` sigue aceptando un archivo individual**; la comprobación "sigue funcionando con las
   demás criaturas" se demuestra por `run` sobre una carpeta mixta (F5-03), que es el camino del
   enunciado.

## Issues

Desglosado en 5 archivos de issue en `plan/F5-errores-carga-resiliente/`, listos para ejecutar en
sesiones TDD (test que falla primero, luego código). Estado inicial: todos `todo`.

| ID | Título | Cubre | Depende de | Tamaño | Paralelizable con | Estado | Archivo de issue |
|----|--------|-------|------------|--------|-------------------|--------|------------------|
| F5-01 | Mapa con carácter desconocido rechazado, con carácter y línea | Enunciado carga de mapas; criterio de salida | F4 | M | — | todo | F5-01-mapa-caracter-desconocido-rechazado.md |
| F5-02 | Batería de archivos inválidos, uno por error de compilación de 2.8 | Catálogo de errores de compilación | F5-01 | M | F5-03 (fixtures disjuntos) | todo | F5-02-bateria-errores-compilacion.md |
| F5-03 | Carga resiliente de punta a punta sobre una carpeta mixta | "Sigue funcionando con los demás" | F5-01 | M | F5-02 | todo | F5-03-carga-resiliente-punta-a-punta.md |
| F5-04 | Errores de ejecución con archivo real y registro del tick | Errores de ejecución de 2.8; registro | — | M | F5-02, F5-03 | todo | F5-04-errores-ejecucion-registro.md |
| F5-05 | Validación acumulativa en `check` y limpieza del compilador | Patrón 6.3 (Either) | F5-02 | M | F5-04 | todo | F5-05-validacion-acumulativa-check.md |

Cinco issues, dentro del rango del MASTER para F5 (3–5).

### Orden y dependencias

- F5-01 primero: es el único cambio de comportamiento de la fase y el de mayor riesgo; conviene
  encontrar pronto cualquier sorpresa.
- F5-02 y F5-03 comparten el estilo de fixtures pero tocan carpetas distintas (`errors/` vs
  `mixed_root/`); pueden ir en cualquier orden.
- F5-04 toca núcleo (interp/sim/event/species) y es independiente de los demás.
- F5-05 cierra: sobre la batería de F5-02 ya creada, añade los casos de acumulación y limpia el
  compilador.

## Cobertura

| Entregable / criterio de salida de la fase | Issues que lo cubren |
|--------------------------------------------|----------------------|
| Todos los errores de la sección 2.8 cubiertos con mensaje y línea | F5-02 (compilación), F5-04 (ejecución) |
| Mapa con carácter desconocido rechazado con carácter y línea | F5-01 |
| Un error de ejecución mata solo a esa criatura y queda en el registro | F5-04 |
| Carga que sigue funcionando con los demás recursos | F5-01, F5-03 |
| Carpeta de archivos inválidos con mensaje y línea esperados | F5-02 |
| Validación acumulativa (patrón 6.3 del documento de patrones) | F5-05 |

## Verificación de salida de la fase

- `cargo test --workspace` → todo en verde, incluida la batería de `fixtures/errors/` y la carga
  mixta de `fixtures/mixed_root/`.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con la misma semilla sobre los cuatro ejemplos del anexo dan la misma historia que
  antes de la fase (determinismo intacto).
- Sesión de verificación recomendada al cierre: recorrer el `README` de la batería y comprobar
  archivo por archivo el mensaje y la línea contra la salida real de `check`.

## Prompt de sesión (copiar/pegar para cada issue)

```text
Implementa el issue plan\F5-errores-carga-resiliente\<ID>-<slug>.md.
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
Escribe primero el test que falle, ejecuta los comandos de "Verificación" y no lo des por
terminado hasta cubrir cada criterio de aceptación con evidencia. Al acabar: actualiza `status`
aquí y en PHASE.md y rellena "Notas de implementación". Si el issue contradice el repo o te
bloquea, para y repórtalo en lugar de improvisar.
```

## Registro de ejecución

<!-- Se rellena al avanzar: resultado por issue, señales de dimensionado (¿grande/pequeño?),
hechos descubiertos que cambian el plan. Lo consume el Modo Revisión. -->
