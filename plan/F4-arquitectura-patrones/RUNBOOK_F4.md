# InstinctRust — Runbook de F4: Arquitectura y patrones de diseño

**Fase:** F4 · Arquitectura y patrones de diseño  
**Carpeta:** `plan\F4-arquitectura-patrones\`  
**Dependencia:** F3 cerrada (31 percepciones, `see`/`name`, 5 constantes, 4 ejemplos del anexo).  
**Línea base:** `cargo test --workspace` (exit 0), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), `cargo fmt --all --check` (exit 0).

---

## 1. Objetivo de F4

Aplicar los patrones de `docs\PATRONES_DISENO_ARQUITECTURA.md` que **no** arreglan bugs: los que reestructuran código correcto para que el proyecto escale y se mantenga. Al terminar:

- El núcleo está desacoplado de la E/S mediante puertos y adaptadores.
- Las criaturas viven en almacenes indexados por identificador (ECS ligero).
- El estado de turno está modelado (patrón State).
- El AST admite operaciones nuevas sin tocar los nodos (Visitor, solo si hay consumidor real).
- La referencia sigue siendo reproducible: misma semilla, misma historia.

**Regla de oro:** ningún patrón cambia comportamiento observable. La puerta de salida de cada issue es que dos corridas con la misma semilla den la misma historia.

---

## 2. Los 8 issues de F4

| ID | Título | Patrón | Depende de | Tamaño | Archivo |
|----|--------|--------|------------|--------|---------|
| F4-01 | Bus de eventos formal y puerto de salida del mundo | Observer / bus + puertos y adaptadores | F3 | M | F4-01-bus-eventos-puerto-salida.md |
| F4-02 | Estado de turno de criatura con patrón State | State | F3 | M | F4-02-estado-turno-creatura-state.md |
| F4-03 | Estrategia de PRNG inyectable en pruebas | Strategy (PRNG) | F3 | S | F4-03-prng-inyectable-strategy.md |
| F4-04 | Almacenes de criaturas indexados por identificador (ECS ligero) | ECS ligero | F3, F4-02 | L | F4-04-almacenes-criaturas-ecs.md |
| F4-05 | Flyweight para especies (y catálogos compartidos) | Flyweight | F4-04 | M | F4-05-flyweight-especies.md |
| F4-06 | Memento del estado de criatura para el turno | Memento / Snapshot | F4-02 | S | F4-06-memento-estado-turno.md |
| F4-07 | Visitor sobre el AST, solo con consumidor real | Visitor (AST) | F3 | M | F4-07-visitor-ast.md |
| F4-08 | Capas y límites de módulos verificados, sin código pasarela | Capas (Layered) | F3, F4-04 | S | F4-08-capas-modulos-limites.md |

### Orden sugerido de ejecución

1. **F4-01**, **F4-02**, **F4-03** — raíces independientes, se pueden hacer en cualquier orden (una por sesión).
2. **F4-04** — depende de F4-02.
3. **F4-05** — depende de F4-04.
4. **F4-06** — depende de F4-02.
5. **F4-07** — al final, solo si hay consumidor real (pretty-printer / type-checker). Si no, se documenta el descarte y no se implementa.
6. **F4-08** — cierra la fase; verifica capas y elimina código pasarela.

> Cada issue es una sesión. Un issue por sesión. No mezclar issues en la misma sesión.

---

## 3. Cómo empezar un issue (TDD obligatorio)

Antes de tocar código, en cada sesión:

1. **Lee el issue entero** (`plan\F4-arquitectura-patrones\<ID>-<slug>.md`), empezando por su sección "Leer primero".
2. **Escribe primero el test que falle** (unitario en `crates\instinct-core\tests\` o de integración en `crates\instinct-cli\tests\`).
3. **Ejecuta el test** y confirma que falla (compila pero el test no pasa, o no compila por un método que no existe).
4. **Aplica la edición quirúrgica** con la herramienta `edit` sobre el archivo fuente.
5. **Ejecuta inmediatamente las pruebas afectadas** (ver sección 5).
6. Si hay errores del compilador o clippy, resuélvelos sin `#[allow>` injustificado.
7. Antes de dar por cerrado el issue: suite completa en verde (`cargo test --workspace`, clippy, fmt).

---

## 4. Comandos obligatorios (Windows / PowerShell 5.1)

Ejecutar desde la raíz del workspace (`D:\Proyectos\InstinctRust`). **No usar `&&`**; encadenar con `;` o pasos separados.

### Antes de empezar cada sesión (línea base)
```powershell
cargo test --workspace
```

### Después de cada cambio (pruebas afectadas)
```powershell
cargo test -p instinct-core
# o, si tocaste la CLI:
cargo test -p instinct-cli
# o un test concreto:
cargo test -p instinct-core --test <nombre_test>
```

### Antes de cerrar un issue (puerta de salida completa)
```powershell
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

### Determinismo (misma semilla, misma historia)
```powershell
cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 7 --ticks 30 --place Dwarf@0,1 --place Ent@5,5 --place Hobbit@10,10 --place Uruk@15,3
# Ejecutar dos veces y comparar salida (byte a byte donde exista comparación)
```

### Compilación de ejemplos del anexo
```powershell
cargo run -p instinct-cli -- check examples\creatures\hobbit.ins
cargo run -p instinct-cli -- check examples\creatures\ent.ins
cargo run -p instinct-cli -- check examples\creatures\dwarf.ins
cargo run -p instinct-cli -- check examples\creatures\uruk.ins
```

---

## 5. Qué verificar antes de marcar un issue como `done`

Cada issue tiene sus propios Criterios de Aceptación (AC). Además, siempre:

- [ ] Todos los AC del issue cumplidos con evidencia (comandos ejecutados y resultado).
- [ ] `cargo test --workspace` → todo en verde (0 failed).
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- [ ] `cargo fmt --all --check` → exit 0.
- [ ] Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo en `crates\instinct-core\src` y `crates\instinct-cli\src`.
- [ ] Cero `println!` en el núcleo (`crates\instinct-core\src`).
- [ ] Cero `f32`/`f64` en el núcleo.
- [ ] Dos corridas con la misma semilla sobre los cuatro ejemplos del anexo dan la misma historia.
- [ ] Sin stubs, TODOs ni código muerto.
- [ ] Cambios de firma declarados en el issue y propagados a los sitios de llamada en el mismo cambio.
- [ ] Actualizar `status` a `done` tanto en el archivo del issue como en la tabla de `PHASE.md`.
- [ ] Rellenar "Notas de implementación" al final del issue (qué comandos ejecutaste, AC por AC, desviaciones).

---

## 6. Qué hacer si te bloqueas

| Bloqueo | Qué hacer |
|---------|-----------|
| El patrón parece exigir que el intérprete escriba al mundo (rompe "el intérprete lee, el mundo escribe") | Para y repórtalo en "Notas de implementación"; no improvises. El intérprete produce datos, el mundo/simulación los publica. |
| `Box<dyn RngSource>` complica `Simulation::new(world: World)` | El RNG viaja dentro del `World`; `Simulation` no lo toca directamente. No uses `RefCell<Rng>` nuevo ni `rand`. |
| `remove_creature` invalida el iterador de ids vivos en `step_tick` | Recolecta primero los ids vivos en un `Vec` y itera sobre esa copia, o marca muertas y depura al final del tick. No uses referencias a `Creature` desde la rejilla. |
| `Rc<Species>` obliga a cambiar muchas firmas que reciben `&Species` | Considera `SpeciesId` + `World::species(id) -> &Species` como alternativa. No clones `Species` por criatura. |
| No hay consumidor real para Visitor (F4-07) | No implements Visitor. Documenta el descarte con motivación y coste; pasa a F4-08. |
| Un patrón cambia comportamiento observable | Para y repórtalo; sale de F4 y se trata como bug en la siguiente fase de estabilidad. |

---

## 7. Criterios de salida de la fase F4

Todos los siguientes deben cumplirse antes de dar por cerrada F4:

- [ ] Los 8 issues en estado `done` (o F4-07 documentado como descartado por falta de consumidor).
- [ ] `cargo test --workspace` → todo en verde.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- [ ] `cargo fmt --all --check` → exit 0.
- [ ] Dos corridas con la misma semilla sobre los cuatro ejemplos del anexo dan la misma historia que antes de F4 (comparación tick a tick).
- [ ] Consumo de memoria y tiempo por tick medidos antes y después de F4, registrados en `PHASE.md#registro-de-ejecución`.
- [ ] Cada issue declara el patrón aplicado, la motivación y el coste asumido.
- [ ] Cero dependencias externas en `instinct-core`; cero `println!` en el núcleo; cero `f32`/`f64`.
- [ ] La API que consume la CLI sigue intacta o quedó actualizada y documentada.
- [ ] `MASTER.md` actualizado con el cierre de F4 en el registro de cambios.

---

## 8. Cierre de un issue

Al terminar cada issue:

1. Cambia `status: todo` → `status: done` en el frontmatter del issue.
2. Actualiza la tabla de `PHASE.md` (columna Estado → `done`).
3. Rellena "Notas de implementación" con: rama, commit, comandos ejecutados y su resultado, AC por AC (PASS/FAIL), desviaciones o heurística.
4. No edites Alcance ni Criterios de aceptación del issue (si están mal, repórtalo antes de cerrar).

---

## 9. Cierre de la fase F4

Cuando todos los issues estén en `done`:

1. Verifica los criterios de salida de la sección 7.
2. Rellena el "Registro de ejecución" en `PHASE.md` con patrón, motivación, coste asumido y mediciones por issue.
3. Actualiza `plan\MASTER.md` con una línea en el registro de cambios: cierre de F4 (8/8 issues) y planeación de F5.
4. Ejecuta una última vez la suite completa: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`.
5. Invoca al subagente `reviewer` para auditar el diff antes de dar por cerrada la fase.

---

## 10. Archivos de referencia

- `plan\F4-arquitectura-patrones\PHASE.md` — objetivos, contratos, decisiones, cobertura, registro de ejecución.
- `plan\MASTER.md` — plan general, contratos entre fases, señales de replanificación.
- `docs\PATRONES_DISENO_ARQUITECTURA.md` — fuente de los patrones aplicados en F4.
- `docs\PROBLEMAS_Y_SOLUCIONES.md` — problemas que F4 resuelve (I3, I6, W5).
- `plan\F3-percepciones-funciones-texto\PHASE.md` — estado de partida de F3 (contratos que F4 respeta).

---

*Runbook generado al abrir F4. Revisar y ajustar al avanzar si surge algún hallazgo que cambie el plan.*
