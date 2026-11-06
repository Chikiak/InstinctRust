# Patrones de Diseño y Arquitectura para InstinctRust

Este documento propone patrones de diseño y arquitectura aplicables al proyecto **InstinctRust** (intérprete y simulador del lenguaje Instinct en Rust). Para cada patrón se indica:

* **Qué es** (definición breve).
* **Qué afecta en el proyecto** (módulos, archivos y responsabilidades tocados).
* **Por qué** (ventajas, desventajas y trade-offs específicos de este proyecto).
* **Cómo se aplicaría** (ejemplo concreto sobre el código existente).

Se respetan las restricciones del núcleo: **sin dependencias externas** en `instinct-core`, **sin `f32`/`f64`** (solo `i64` y texto), **sin `println!` en el núcleo** (el mundo emite eventos), **sin `rand`**, **identificadores estables** (la rejilla guarda IDs, no referencias), y **cero panics con entrada inválida**.

---

## 1. Arquitectura general

### 1.1. Arquitectura en capas (Layered Architecture)

**Qué es:** Organiza el código en capas estrictas donde cada capa solo depende de la inferior. Flujo sugerido:

```
CLI (instinct-cli)
  └─► Compilación (compile + expr + lexer)
        └─► Intérprete (interp + program + value)
              └─► Mundo / Simulación (world + sim + species + resources + actions + perception)
                    └─► Núcleo de tipos (diagnostic + event + rng)
```

**Qué afecta:** `crates/instinct-core/src/lib.rs` (orden de módulos), `crates/instinct-cli/src/main.rs`, y los límites entre `compile`, `interp`, `world` y `sim`.

**Por qué:**
* Ventaja: claridad de dependencias y facilidad de probar cada capa aisladamente (TDD).
* Desventaja: puede generar código "pasarela" (capa que solo reenvía llamadas).
* Aplicación en InstinctRust: ya existe parcialmente, pero `Simulation` conoce demasiado de `World` y viceversa (desacoplo de catálogos de terrenos/objetos, ver problema W2). Capas claras ayudarían a eliminar ese acoplamiento bidireccional.

### 1.2. Puertos y adaptadores (Hexagonal / Clean Architecture)

**Qué es:** El núcleo de negocio (reglas del lenguaje Instinct, simulación de ticks) no conoce detalles externos (sistema de archivos, argumentos de CLI, impresión de pantalla). Los detalles entran mediante adaptadores a través de puertos (traits).

**Qué afecta:** `crates/instinct-cli/src/main.rs` (entrada/salida), `crates/instinct-core/src/event.rs` (salida del mundo), `crates/instinct-core/src/resources.rs` (carga de archivos).

**Por qué:**
* Ventaja: `instinct-core` permanece puro, sin `println!`, sin E/S de archivos, y fácil de testear. La CLI es solo un adaptador de consola; otro día se podría añadir una GUI o servidor web sin tocar el núcleo.
* Desventaja: más abstracciones (traits) y algo de boilerplate.
* Aplicación en InstinctRust: el comentario en `lib.rs` ya dice "the world emits events, never prints". Se formalizaría definiendo un trait `WorldOutput` o similar para eventos, y la CLI lo implementaría imprimiendo a `stdout`/`stderr`.

---

## 2. Patrones para el lexer y parser

### 2.1. Descenso recursivo (Recursive Descent)

**Qué es:** Cada regla de la gramática es una función que llama a otras funciones para reglas anidadas.

**Qué afecta:** `crates/instinct-core/src/expr.rs` (struct `Parser` y métodos `parse_or`, `parse_and`, etc.).

**Por qué:** Ya está implementado. Es adecuado para una gramática LL(1) sencilla como la de Instinct. No se recomienda cambiarlo, pero sí corregir los `unwrap` (problema P1) y añadir recuperación de errores (problema P6).

### 2.2. Patrón Visitor sobre el AST

**Qué es:** Separar la estructura del AST (`Expr`, `Binary`, `Unary`, `LiteralInt`, etc.) de las operaciones que se le aplican (evaluación, impresión, optimización, análisis estático).

**Qué afecta:** `crates/instinct-core/src/expr.rs` (trait `Expr` y structs de nodos), `crates/instinct-core/src/interp.rs` (evalúa expresiones).

**Por qué:**
* Ventaja: se pueden añadir nuevas operaciones sobre el AST sin modificar los nodos (imprimir, type-checker, optimizador de constantes).
* Desventaja: en Rust requiere `dyn` y `Visitor` con `visit_*` para cada variante, o usar macros; añade complejidad si el AST es pequeño y estable.
* Aplicación en InstinctRust: hoy `Expr::eval` está dentro del trait `Expr`. Si se quiere un type-checker o un pretty-printer, convendría extraer a un trait `ExprVisitor` separado.

### 2.3. Patrón Composite

**Qué es:** Tratar expresiones simples y compuestas de forma uniforme mediante una interfaz común (`Expr`).

**Qué afecta:** `crates/instinct-core/src/expr.rs` (trait `Expr`, `Box<dyn Expr>`).

**Por qué:** Ya está aplicado. Permite árboles de expresiones arbitrarios. Es el patrón correcto para el AST y no requiere cambios.

---

## 3. Patrones para el intérprete

### 3.1. Patrón Interpreter (máquina de instrucciones)

**Qué es:** El programa compilado es una secuencia de instrucciones (`Line`) que una máquina virtual ejecuta mediante un contador de programa (`pc`).

**Qué afecta:** `crates/instinct-core/src/program.rs` (`Line`, `Program`), `crates/instinct-core/src/interp.rs` (`execute_line`, `execute_turn`).

**Por qué:** Ya está aplicado. Es adecuado para un lenguaje sencillo de ejecución por turno. Mejoras posibles: convertir `Line` a un enum con datos propios en lugar de `Box<dyn Expr>` para acciones, evitando doble indirección.

### 3.2. Patrón State (estado de turno de criatura)

**Qué es:** Un objeto cambia su comportamiento según su estado interno (ej. `Awake`, `Sleeping`, `Dead`).

**Qué afecta:** `crates/instinct-core/src/interp.rs` (`CreatureTurnState`, `sleep_turns`), `crates/instinct-core/src/world.rs` (`Creature.sleep_turns`).

**Por qué:**
* Ventaja: encapsula la lógica de "estoy dormido N turnos" sin condiciónales esparcidos en `execute_turn`.
* Desventaja: sobre-dimensionado si `sleep_turns` es solo un contador entero.
* Aplicación en InstinctRust: `CreatureTurnState` ya lleva `sleep_turns: u32`. Se podría modelar como enum `CreatureState { Awake, Sleeping(u32), Dead }` y delegar el comportamiento a ese estado, eliminando la dispersión de `if sleep_turns > 0` en `sim` e `interp`.

### 3.3. Patrón Command (acciones del lenguaje)

**Qué es:** Cada acción (`wait`, `move`, `say`, `attack`, etc.) es un objeto comando con `execute()` que retorna el efecto y si consume turno.

**Qué afecta:** `crates/instinct-core/src/actions.rs` (trait `Action`, `Wait`, `Move`, `Say`, etc.), `crates/instinct-core/src/interp.rs` (`PendingAction`).

**Por qué:** Ya está aplicado mediante el trait `Action`. Encapsula bien la variabilidad de acciones. Se podría mejorar aplicando **Null Object** para acciones inválidas en lugar de devolver `Diagnostic` por cada error de aridad.

### 3.4. Patrón Memento o Snapshot (estado de criatura para simulación)

**Qué es:** Guardar una copia del estado relevante de una criatura antes de ejecutar su turno, para poder restaurar o persistir cambios atómicamente.

**Qué afecta:** `crates/instinct-core/src/sim.rs` (`run_creature_turn` clona la criatura), `crates/instinct-core/src/world.rs` (`CreatureSnapshot`).

**Por qué:** Ya se hace un `clone` manual en `sim.rs`. Un Memento formalizado evitaría copias incompletas y permitiría deshacer si el turno falla a mitad.

---

## 4. Patrones para mundo y simulación

### 4.1. Entity Component System (ECS) ligero

**Qué es:** Las entidades son solo IDs (`CreatureId`); los datos viven en almacenes separados (componentes); los sistemas procesan los almacenes.

**Qué afecta:** `crates/instinct-core/src/world.rs` (`struct Creature`, `Vec<Creature>`, `content: Vec<Vec<CellContent>>`), `crates/instinct-core/src/sim.rs` (iteración sobre criaturas).

**Por qué:**
* Ventaja: la rejilla ya guarda IDs (no referencias), lo cual es compatible con ECS. Facilita agregar nuevos componentes (ej. `Inventario`, `EfectoEstado`) sin modificar `Creature`.
* Desventaja: requiere reescribir `World` para usar almacenes indexados por ID en lugar de `Vec<Creature>` con búsqueda lineal (problema W5).
* Aplicación en InstinctRust: se puede adoptar parcialmente manteniendo `CreatureId(usize)` y moviendo `health`, `age`, `variables` a `HashMap<CreatureId, Componente>` o `Vec<Option<Componente>>`.

### 4.2. Flyweight (pesos ligeros) para especies, terrenos y objetos

**Qué es:** Compartir datos inmutables y comunes entre muchas instancias (todas las criaturas de una especie comparten `Species`; todos los terrenos de hierba comparten `Terrain`).

**Qué afecta:** `crates/instinct-core/src/species.rs` (`Species`), `crates/instinct-core/src/resources.rs` (`Terrain`, `Object`), `crates/instinct-core/src/world.rs` (`Creature.species: Species` se clona por criatura).

**Por qué:**
* Ventaja: reduce memoria al no clonar `Species` completo por cada criatura; solo se guarda `SpeciesId` o referencia compartida (`Rc<Species>`).
* Desventaja: si `Species` muta (programa, salud base), hay que separar estado intrínseco y extrínseco.
* Aplicación en InstinctRust: `Creature.species` es `Species` (clonado). Cambiar a `Rc<Species>` o `SpeciesId` reduciría copias y haría el mundo más ligero.

### 4.3. Observer / Event Bus (emisión de eventos)

**Qué es:** El mundo publica eventos (`Said`, `Died`, `Damaged`, `Roared`, `RuntimeError`) y suscriptores externos (CLI, tests, posible GUI) reaccionan sin que el núcleo sepa quién los consume.

**Qué afecta:** `crates/instinct-core/src/event.rs` (`Event`), `crates/instinct-core/src/world.rs` (`emit_event`, `drain_events`), `crates/instinct-cli/src/main.rs` (imprime eventos).

**Por qué:** Ya está aplicado parcialmente (`World.events`, `drain_events`). Se formalizaría como un bus de eventos donde `World` no conoce a la CLI. Esto cumple la restricción de "sin `println!` en el núcleo".

### 4.4. Object Pool para criaturas activas

**Qué es:** Reutilizar estructuras de criatura en lugar de allocar/deallocar cada tick.

**Qué afecta:** `crates/instinct-core/src/world.rs` (`place_creature`, `remove_creature`), `crates/instinct-core/src/sim.rs` (muertes y nacimientos).

**Por qué:** Solo aplica si hay miles de criaturas y alta rotación. Para el alcance actual de InstinctRust es prématuro; se menciona como opción futura si surge un problema de rendimiento medido.

---

## 5. Patrones para acciones, percepciones y catálogos

### 5.1. Strategy para acciones y percepciones

**Qué es:** Definir una familia de algoritmos (`Action`, `Perception`), encapsular cada uno y hacerlos intercambiables.

**Qué afecta:** `crates/instinct-core/src/actions.rs` (trait `Action`, `Catalog`), `crates/instinct-core/src/perception.rs` (trait `Perception`, `PerceptionCatalog`).

**Por qué:** Ya está aplicado con traits y `dyn`. Es el patrón correcto para permitir acciones y percepciones definidas por el usuario sin modificar el núcleo.

### 5.2. Registry / Catalog para acciones y percepciones por nombre

**Qué es:** Un catálogo central que mapea nombres a instancias de estrategia, validando existencia y aridad.

**Qué afecta:** `crates/instinct-core/src/actions.rs` (`Catalog::resolve`), `crates/instinct-core/src/perception.rs` (`PerceptionCatalog::resolve`), `crates/instinct-core/src/compile.rs` (validación de nombres de acción en `parse_action_call`).

**Por qué:** Ya existe. Se podría mejorar agregando **Validación temprana** en tiempo de compilación (todos los nombres de acción usados en el programa `.ins` deben estar en el catálogo) en lugar de solo en parseo de línea individual.

### 5.3. Null Object para acciones inválidas

**Qué es:** Devolver un objeto "nulo" en lugar de `None` o error cuando no se encuentra una acción, con comportamiento seguro (no hace nada, no consume turno).

**Qué afecta:** `crates/instinct-core/src/actions.rs` (`Catalog::resolve` devuelve `Result`), `crates/instinct-core/src/compile.rs` (error "acción desconocida").

**Por qué:** Evita propagar `Result` por todas partes para acciones opcionales. Sin embargo, en InstinctRust una acción desconocida es un error real del programa, así que `Result` es más apropiado que Null Object.

---

## 6. Patrones para manejo de errores y diagnósticos

### 6.1. Result / Option idiomático de Rust (ya usado)

**Qué es:** Usar `Result<T, E>` y `Option<T>` para errores y ausencias, sin `unwrap`/`expect` sobre datos externos.

**Qué afecta:** Todo `instinct-core`, especialmente `lexer.rs` (`LexError`), `expr.rs` (`ParseError`), `interp.rs` (`(Diagnostic, usize)`), `world.rs` (`Result<_, Diagnostic>`).

**Por qué:** Ya es el estándar del proyecto. Los problemas detectados son excepciones (`unwrap` en parser, `expect` en `draw`, `unwrap_or(0)` en `interp`) que deben eliminarse.

### 6.2. Error Chain / Envoltura de errores

**Qué es:** Un tipo de error que puede contener la causa original y contexto (archivo, línea).

**Qué afecta:** `crates/instinct-core/src/diagnostic.rs` (`Diagnostic`), `crates/instinct-core/src/lexer.rs` (`LexError`), `crates/instinct-core/src/expr.rs` (`ParseError`).

**Por qué:** Ayudaría a unificar `LexError`, `ParseError` y errores de ejecución bajo un mismo `Diagnostic` con `file` y `line`, resolviendo el problema L1 y la dispersión de tipos de error.

### 6.3. Either / Validación acumulativa

**Qué es:** En lugar de fallar en el primer error, acumular todos los errores de validación y devolverlos juntos.

**Qué afecta:** `crates/instinct-core/src/compile.rs` (fase de header y cuerpo acumula diagnósticos, pero `parse_action_call` pierde errores de args), `crates/instinct-core/src/lexer.rs` (primer error aborta).

**Por qué:** Útil para `check` en la CLI: el usuario prefiere ver todos los errores de su programa `.ins` de una vez, no uno por uno en cada ejecución.

---

## 7. Patrones para CLI y E/S

### 7.1. Command para subcomandos de CLI

**Qué es:** Cada subcomando (`run`, `check`, `--help`) es un objeto comando con `execute() -> i32`.

**Qué afecta:** `crates/instinct-cli/src/main.rs` (`execute_run`, `execute_check`, dispatch en `main`).

**Por qué:** Reemplazaría el `match` manual en `main` y el parsing manual de args. Facilita añadir subcomandos y pruebas unitarias por comando.

### 7.2. Builder para argumentos de ejecución (`RunArgs`)

**Qué es:** Construir `RunArgs` paso a paso con validación de campos obligatorios y rangos.

**Qué afecta:** `crates/instinct-cli/src/main.rs` (`parse_run`, `RunArgs`).

**Por qué:** Permite validar `--ticks >= 0`, `--every > 0`, existencia de `--root`, y rechazar flags desconocidos antes de construir el struct (resuelve problemas C1, C2, C3).

### 7.3. Separación de streams (stdout vs stderr)

**Qué es:** La salida de simulación (mapa, eventos) va a `stdout`; diagnósticos y errores van a `stderr`.

**Qué afecta:** `crates/instinct-cli/src/main.rs` (impresión de eventos y mapa).

**Por qué:** Evita mezclar contenido de simulación con mensajes de error, lo que complica el parsing de salida por scripts externos (problema C5).

---

## 8. Patrones para determinismo y aleatoriedad

### 8.1. Deterministic PRNG + Strategy

**Qué es:** Un generador de números pseudoaleatorios con semilla fija, intercambiable mediante trait, para que la misma semilla produzca la misma historia.

**Qué afecta:** `crates/instinct-core/src/rng.rs` (PRNG propio), `crates/instinct-core/src/world.rs` (`rng: RefCell<Rng>`), `crates/instinct-core/src/sim.rs` (`shuffle` con `world.rng`).

**Por qué:** Ya existe un PRNG propio determinista. El patrón Strategy permitiría inyectar un RNG de prueba en tests sin depender del estado global del mundo.

---

## 9. Matriz resumen: patrón → capa → problema que resuelve

| Patrón | Capa / módulo | Problema que resuelve | Impacto | Recomendación |
|--------|---------------|----------------------|---------|---------------|
| Capas (Layered) | `lib.rs`, `main.rs` | Acoplamiento entre compilación, intérprete y simulación | Medio | Adoptar como guía de organización |
| Puertos y adaptadores | `main.rs`, `event.rs` | E/S y `println!` fuera del núcleo | Alto | Formalizar trait de salida de eventos |
| Descenso recursivo | `expr.rs` | Parser existente | Bajo | Mantener, corregir `unwrap` |
| Visitor (AST) | `expr.rs`, `interp.rs` | Añadir operaciones al AST sin tocar nodos | Medio | Considerar si se añade type-checker/pretty-printer |
| Composite (AST) | `expr.rs` | Árboles de expresiones uniforme | Bajo | Ya aplicado |
| Interpreter (máquina de instrucciones) | `program.rs`, `interp.rs` | Ejecución por turno | Bajo | Ya aplicado |
| State (turno de criatura) | `interp.rs`, `world.rs` | Lógica de `sleep_turns` dispersa | Medio | Considerar enum `CreatureState` |
| Command (acciones) | `actions.rs`, `interp.rs` | Variabilidad de acciones | Bajo | Ya aplicado |
| Memento/Snapshot | `sim.rs`, `world.rs` | Copia atómica de estado de turno | Medio | Útil si se añade deshacer/transacciones |
| ECS ligero | `world.rs`, `sim.rs` | Búsqueda lineal O(n) y almacenamiento de criaturas | Alto | Refactorizar `Vec<Creature>` a almacenes por ID |
| Flyweight | `species.rs`, `world.rs` | Clonar `Species` por cada criatura | Medio | Cambiar a `Rc<Species>` o `SpeciesId` |
| Observer / Event Bus | `event.rs`, `world.rs`, `main.rs` | Mundo emite eventos sin conocer al consumidor | Alto | Formalizar y respetar "no println en núcleo" |
| Strategy (acciones/percepciones) | `actions.rs`, `perception.rs` | Intercambiabilidad de comportamientos | Bajo | Ya aplicado |
| Catalog/Registry | `actions.rs`, `perception.rs`, `compile.rs` | Resolución por nombre y aridad | Bajo | Ya aplicado |
| Result/Option idiomático | Todo el núcleo | Manejo de errores sin panics | Alto | Eliminar `unwrap`/`expect` restantes |
| Error Chain (Diagnostic unificado) | `diagnostic.rs`, `lexer.rs`, `expr.rs` | Varios tipos de error dispersos | Alto | Unificar bajo `Diagnostic` con `file:line` |
| Either / Validación acumulativa | `compile.rs`, `lexer.rs` | Mostrar todos los errores de una vez | Medio | Útil para `check` en CLI |
| Command (subcomandos CLI) | `main.rs` | Dispatch manual y parsing frágil | Medio | Reemplazar `match` en `main` |
| Builder (RunArgs) | `main.rs` | Validación de `--ticks`, `--every`, `--root` | Alto | Validar antes de construir |
| Separación de streams | `main.rs` | Mezcla stdout/stderr | Medio | Ejecutar eventos a stdout, errores a stderr |
| Deterministic PRNG Strategy | `rng.rs`, `world.rs`, `sim.rs` | Reproducibilidad y tests | Bajo | Ya aplicado, inyectar RNG en tests |

---

## 10. Hoja de adopción sugerida (por fases)

### Fase 1 — Estabilidad y cero panics (inmediato)
* Patrón **Result/Option idiomático**: eliminar `unwrap`/`expect` en `expr.rs`, `interp.rs`, `world.rs`.
* Patrón **Error Chain / Diagnostic unificado**: unificar `LexError`, `ParseError` bajo `Diagnostic` con `file` y `line`.
* Patrón **Builder (RunArgs)**: validar `--ticks >= 0`, `--every > 0`, rechazar flags desconocidos en CLI.

### Fase 2 — Arquitectura limpia (corto plazo)
* Patrón **Puertos y adaptadores**: trait de salida de eventos; `instinct-core` sin E/S ni `println!`.
* Patrón **Capas**: clarificar dependencias entre `compile`, `interp`, `world`, `sim`.
* Patrón **Observer / Event Bus**: formalizar `drain_events` como bus.

### Fase 3 — Escalabilidad y mantenibilidad (medio/largo plazo)
* Patrón **ECS ligero**: almacenes de criaturas indexados por `CreatureId`, eliminar búsqueda lineal O(n).
* Patrón **Flyweight**: `Rc<Species>` o `SpeciesId` en `Creature`.
* Patrón **State**: `CreatureState { Awake, Sleeping(u32), Dead }`.
* Patrón **Visitor (AST)**: si se añade type-checker, pretty-printer u optimizador.

---

## Conclusión

InstinctRust ya aplica correctamente varios patrones clave (**Composite** para el AST, **Command** para acciones, **Strategy** para percepciones, **Observer** ligero para eventos, **Descenso recursivo** para el parser). Las mayores oportunidades de mejora están en:

1. **Eliminar panics y fallos silenciosos** (`unwrap`, `expect`, `unwrap_or`) — aplicar `Result/Option` idiomático.
2. **Unificar errores** bajo un `Diagnostic` con `file` y `line`.
3. **Desacoplar el núcleo de E/S** mediante puertos y adaptadores, respetando la regla de "sin `println!` en el núcleo".
4. **Mejorar el almacenamiento de criaturas** hacia un ECS ligero o al menos almacenes indexados, eliminando búsquedas lineales.

Cada patrón puede adoptarse de forma incremental sin reescribir el proyecto de un solo golpe.
