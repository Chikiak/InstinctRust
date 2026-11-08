---
phase: F4
title: Arquitectura y patrones de diseño
status: todo
master: ../MASTER.md
fuentes: ../../docs/PATRONES_DISENO_ARQUITECTURA.md
issues: pendientes de desglosar al abrir la fase
---

# F4 · Arquitectura y patrones de diseño

## Objetivo y entregable

Aplicar los patrones de `docs/PATRONES_DISENO_ARQUITECTURA.md` que **no** arreglan bugs: los que
reestructuran código correcto para que el proyecto escale y se mantenga. Al terminar, la
arquitectura del núcleo está desacoplada de la entrada y salida, las criaturas viven en almacenes
indexados por identificador, el estado de turno está modelado, el AST admite operaciones nuevas sin
tocar los nodos, y la referencia sigue siendo reproducible: misma semilla, misma historia.

Entregable observable: el mismo comportamiento que al entrar, medido; una arquitectura que la
ventana (F7) pueda consumir sin cambios en el núcleo; y un registro de qué patrón se aplicó, con
qué motivación y a qué coste.

## Estado de partida

- **F2 completada**: cero pánicos con entrada inválida, `Diagnostic` único con archivo y línea,
  contratos del lexer, parser, intérprete y mundo probados, catálogos con un solo dueño, CLI
  validada y con flujos separados.
- **F3 completada**: percepciones completas, `see`/`name`, constantes y operaciones con texto; los
  cuatro ejemplos del anexo corren.
- Los patrones que el documento marca como "ya aplicado" siguen aplicados: Composite sobre el AST,
  Command/Strategy para acciones y percepciones, Registry/Catalog por nombre, descenso recursivo,
  Interpreter con `pc`, Observer ligero (`World::events` + `drain_events`), PRNG determinista
  propio.
- Problemas que F4 resuelve, según la tabla de cobertura de F2: I3 e I6 del intérprete y W5 del
  mundo; más los patrones de valor que no aparecen como problema en el documento.

## Contratos compartidos

- **Ningún patrón cambia comportamiento observable.** La puerta de salida de cada issue es que dos
  corridas con la misma semilla den la misma historia, byte a byte donde la comparación exista.
- **Ningún patrón rompe la API del núcleo** que consumen la CLI (F2-08) y consumirá la ventana
  (F7), salvo que el issue lo declare y actualice los sitios de llamada en el mismo cambio.
- **Cero dependencias externas** en `crates/instinct-core`; nada de `f32`/`f64`; nada de
  `println!` en el núcleo; identificadores estables en la rejilla.
- **El intérprete lee y el mundo escribe** no se revisa: si un patrón parece exigir lo contrario,
  el issue se para y se reporta.
- **Patrón por issue, con motivación escrita.** Cada issue dice qué patrón aplica, qué problema o
  capacidad de `docs/PATRONES_DISENO_ARQUITECTURA.md` cubre, y qué coste asume.

## Decisiones de la fase

1. **No se reabren los patrones ya aplicados** salvo para formalizarlos donde un consumidor externo
   lo exige: el Observer ligero se formaliza como bus (I6) y los puertos y adaptadores se hacen
   explícitos, ambos pensando en la ventana de F7.
2. **ECS y Flyweight se aplican pese a no tener medición que los exija** (problemas W5 y la
   sección 4.2 del documento). Se acepta el coste de refactor como aprendizaje y como
   desbloqueo de la ventana y del banco; queda anotado en el issue, y F9 mide después. Si F9
   demostrara que no hacían falta, queda registrado como coste asumido, no como error.
3. **State (`CreatureState`) modela el ciclo de turno de la criatura**, incluido el sueño que hoy
   gestiona `sim.rs` desde fuera (problema I3). `Dead` forma parte del estado solo si el issue
   puede justificarlo con un uso real; en caso contrario se modelan `Awake` y `Sleeping`.
4. **Visitor sobre el AST se aplica solo si llega con un consumidor real** (impresor de expresiones
   o type-checker). Sin consumidor, el patrón no entra: se anota como candidato y no se implementa.
5. **El PRNG pasa a ser inyectable** (Strategy) para que los tests de simulación no dependan del
   estado global del mundo; la secuencia con semilla fija no cambia.
6. **La validación acumulativa (Either) no va en esta fase**: es capacidad de errores y vive en F5
   con el catálogo de errores y la carga resiliente.
7. **Object Pool se descarta** (el documento lo marcaba como prematuro) y queda anotado aquí para
   no volver a proponerlo sin una medición que lo pida.

## Issues

El desglose se hace al abrir la fase, cuando los contratos de F2 y F3 estén fijos en el repo. La
tabla siguiente es la asignación esperada de trabajo, no los ficheros de issue:

| ID | Título | Problema / patrón que cubre | Depende de | Tamaño previsto |
|----|--------|-----------------------------|------------|-----------------|
| F4-01 | Bus de eventos formal y puerto de salida del mundo | I6 (Observer / bus), 1.2 y 4.3 del documento | F3 | M |
| F4-02 | Estado de turno de criatura con patrón State | I3, sección 3.2 | F3 | M |
| F4-03 | Estrategia de PRNG inyectable en pruebas | Sección 8.1 | F3 | S |
| F4-04 | Almacenes de criaturas indexados por identificador (ECS ligero) | W5, sección 4.1 | F3, F4-02 | L |
| F4-05 | Flyweight para especies (y catálogos compartidos) | Sección 4.2 | F4-04 | M |
| F4-06 | Memento del estado de criatura para el turno | Sección 3.4 | F4-02 | S |
| F4-07 | Visitor sobre el AST, solo con consumidor real | Sección 2.2 | F3 | M (condicionado por la decisión 4) |
| F4-08 | Capas y límites de módulos verificados, sin código pasarela | Sección 1.1 | F3 | S |

Notas de dimensionado: ocho issues previstos, dentro del límite de la fase; si F4-04 (ECS) se
divide en dos al abrir la fase, la fase pasa a nueve y se separa el Flyweight a una fase posterior.

### Orden y dependencias (esquema, se concreta al abrir la fase)

- F4-01 y F4-03 son raíces independientes: tocan eventos y aleatoriedad, nadie depende de ellos.
- F4-02 precede a F4-04 (el almacén indexado necesita el estado modelado) y a F4-06 (el Memento
  captura ese estado).
- F4-04 precede a F4-05 (Flyweight) y condiciona F4-01 si el bus publica alta y baja de criaturas.
- F4-07 va al final y solo si tiene consumidor.
- F4-08 cierra: verifica que las capas quedaron limpias y que no apareció código pasarela.

## Cobertura

| Elemento de `docs/PATRONES_DISENO_ARQUITECTURA.md` | Dónde se cubre |
|----------------------------------------------------|----------------|
| 1.1 Capas | F4-08 (y la disciplina de F2) |
| 1.2 Puertos y adaptadores | F4-01 |
| 2.1 Descenso recursivo | Ya aplicado; no se reabre |
| 2.2 Visitor sobre el AST | F4-07 (condicionado) |
| 2.3 Composite | Ya aplicado; no se reabre |
| 3.1 Interpreter / máquina de instrucciones | Ya aplicado; no se reabre |
| 3.2 State | F4-02 |
| 3.3 Command (acciones) | Ya aplicado; F4-04 no lo cambia |
| 3.4 Memento / Snapshot | F4-06 |
| 4.1 ECS ligero | F4-04 |
| 4.2 Flyweight | F4-05 |
| 4.3 Observer / Event Bus | F4-01 |
| 4.4 Object Pool | Descartado (decisión 7) |
| 5.1 Strategy (acciones y percepciones) | Ya aplicado |
| 5.2 Registry / Catalog | Ya aplicado |
| 5.3 Null Object para acciones inválidas | Descartado: una acción desconocida es error real (decisión tomada en F0) |
| 6.1 Result / Option idiomático | Cerrado en F2 |
| 6.2 Error Chain | Cerrado en F2 |
| 6.3 Either / validación acumulativa | F5 |
| 7.1 Command (subcomandos CLI) | Cerrado en F2 |
| 7.2 Builder (RunArgs) | Cerrado en F2 |
| 7.3 Separación de streams | Cerrado en F2 |
| 8.1 Deterministic PRNG + Strategy | F4-03 |

## Verificación de salida de la fase

- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con la misma semilla sobre los cuatro ejemplos del anexo dan la misma historia que
  antes de la fase (comparación tick a tick, no solo mapa final).
- Consumo de memoria y tiempo por tick medidos antes y después, registrados en el informe.
- [ ] Cada issue declara el patrón aplicado, la motivación y el coste asumido.
- [ ] Cero dependencias externas en `instinct-core`; cero `println!` en el núcleo; cero `f32`/`f64`.
- [ ] La API que consume la CLI sigue intacta o quedó actualizada y documentada.

## Prompt de sesión (copiar/pegar cuando existan los issues)

```text
Implementa el issue plan\F4-arquitectura-patrones\<ID>-<slug>.md.
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
Esta fase aplica patrones sin cambiar comportamiento: la puerta de salida de cada issue
incluye el determinismo con semilla fija.
Ejecuta los comandos de "Verificación" y no lo des por terminado hasta cubrir cada
criterio de aceptación con evidencia. Al acabar: actualiza `status` y rellena
"Notas de implementación". Si el issue contradice el repo o te bloquea, para y repórtalo
en lugar de improvisar.
```

## Registro de ejecución

<!-- Se rellena al avanzar: patrón por patrón, motivación, coste asumido, mediciones. -->
