---
id: F4-08
title: Capas y límites de módulos verificados, sin código pasarela
phase: F4
status: todo
depends_on: [F4-04]
size: S
---

# F4-08 · Capas y límites de módulos verificados, sin código pasarela

## Objetivo

Verificar que las capas del núcleo respetan el orden de dependencias de la arquitectura en capas (sección 1.1 de `docs/PATRONES_DISENO_ARQUITECTURA.md`):

```
CLI (instinct-cli)
  └─► Compilación (compile + expr + lexer)
        └─► Intérprete (interp + program + value)
              └─► Mundo / Simulación (world + sim + species + resources + actions + perception)
                    └─► Núcleo de tipos (diagnostic + event + rng)
```

y eliminar código pasarela (capas que solo reenvían llamadas sin añadir valor). También resuelve W2 (desacoplo de catálogos Mundo vs Simulación: `Simulation` es dueño y `World` accede a través de referencias/clones consistentes, sin reservas desincronizadas). Es el issue de cierre de F4.

## Alcance

**Haz**
- Revisar `crates/instinct-core/src/lib.rs` (orden de módulos) y los límites entre `compile`, `expr`, `interp`, `program`, `value`, `world`, `sim`, `species`, `resources`, `actions`, `perception`, `rng`, `event`, `diagnostic`. Cada capa solo depende de las inferiores; no hay dependencias hacia arriba ni circulares.
- W2: unificar el propietario de `terrains` y `objects` para que `World` y `Simulation` no mantengan copias desincronizadas. `Simulation` es dueño de los catálogos (o `World` lo es y `Simulation` accede por referencia consistente); `end_of_tick_terrain_and_objects` regenera sobre el mismo catálogo que `strike_cell`/`damage_cell_content` mutan. Las reservas de terreno y objeto no se desincronizan entre `World` y `Simulation`.
- Eliminar código pasarela: funciones/capas que solo reenvían a otra sin añadir lógica (documentar cuáles se eliminan y cuáles se mantienen como interfaz estable de la API del núcleo).
- No cambiar la API pública que consume la CLI (`run`, `check`, `Simulation::new`, `step_tick`, `World::from_map`, etc.) sin declararlo y actualizar `main.rs` en el mismo cambio.
- Mismo comportamiento: misma semilla, misma historia; consumo de memoria y tiempo por tick no empeoran (medido antes y después, decisión de salida de la fase).
- Pruebas: reservas de terreno consistentes entre World y Simulation; orden de capas sin dependencias circulares (compile pasa sin importar world, etc.); suite completa en verde.

**No hagas**
- No rompas la API del núcleo que consume la CLI (F2-08) ni la que consumirá F7 sin declararlo y actualizar los sitios de llamada.
- No introduzcas `unwrap`/`expect`/`panic!` sobre datos de entrada.
- No añadas dependencias externas en `instinct-core`.
- No hagas refactor especulativo: solo verifica capas y elimina pasarela real.

### Archivos previstos
- `crates/instinct-core/src/lib.rs` — verificar orden de módulos; `pub mod` solo lo necesario.
- `crates/instinct-core/src/sim.rs` — modificar — `Simulation` dueño de catálogos o acceso consistente a los de `World` (W2).
- `crates/instinct-core/src/world.rs` — modificar — acceso a terrenos/objetos consistente con `Simulation`.
- `crates/instinct-core/src/resources.rs` — verificar que los cargadores dejan catálogos consistentes.
- `crates/instinct-cli/src/main.rs` — modificar solo si cambia la API del núcleo (declararlo).
- `crates/instinct-core/tests/world_tests.rs` — modificar — reservas consistentes World/Simulation.
- `crates/instinct-core/tests/sim_tests.rs` — modificar — end_of_tick sobre el catálogo único.
- `crates/instinct-cli/tests/cli_run.rs` — verificar que `run` sigue igual.

## Criterios de aceptación

- [ ] **AC1** Las dependencias de módulos respetan el orden de capas: `compile`/`expr`/`lexer` no dependen de `world`/`sim`; `interp` no depende de CLI; `world`/`sim` no dependen de `compile`. Verif: `cargo check --workspace` y revisión de `use`.
- [ ] **AC2** W2 resuelto: un único dueño consistente de `terrains`/`objects`; `end_of_tick_terrain_and_objects` y `damage_cell_content` actúan sobre el mismo catálogo; reservas no se desincronizan. Verif: `test_terrain_reserve_consistent_world_simulation` | `cargo test -p instinct-core --test world_tests`.
- [ ] **AC3** Código pasarela identificado y eliminado (o justificado como interfaz de API estable). Verif: notas de implementación.
- [ ] **AC4** La API que consume la CLI sigue intacta (o quedó actualizada y documentada en el mismo issue). Verif: `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC5** Consumo de memoria y tiempo por tick medidos antes y después de F4 (esta issue cierra la medición): no empeoran de forma significativa. Verif: registro en `PHASE.md#registro-de-ejecución`.
- [ ] **AC6** Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Suite completa + clippy + fmt en verde.

## Contexto

**Ya existe**: `lib.rs` con el orden de módulos; `World` y `Simulation` cada uno con sus propios `terrains`/`objects` (W2 abierto: `Simulation` mantiene los suyos, `World` clona los suyos, `end_of_tick_terrain_and_objects` regenera sobre los de `Simulation` desincronizando reservas). `Simulation { world: World }` — `Simulation` envuelve a `World`, por lo que los catálogos viven dentro de `World` y `Simulation` accede a través de `world`.

**Contratos que debes respetar**: API del núcleo estable para CLI; `Simulation::new(world)`, `step_tick()`, `tick()`, `world()` siguen funcionando; fin de tick completo; cero pánicos en el núcleo.

**Ojo**: como `Simulation { world: World }`, los catálogos de terrenos/objetos ya viven en `World` y `Simulation` los lee a través de `world`. Si `sim.rs` mantenía copias separadas, unifícalas. Documenta en el cierre qué se encontró y qué se eliminó.

### Leer primero
- `crates/instinct-core/src/lib.rs` — orden de módulos.
- `crates/instinct-core/src/world.rs` — `terrains`, `objects`, `World::from_map`.
- `crates/instinct-core/src/sim.rs` — `Simulation`, `step_tick`, `end_of_tick_terrain_and_objects`, `end_of_tick_creatures`.
- `crates/instinct-cli/src/main.rs` — cómo la CLI crea `World` y `Simulation`.

## Especificación

```
lib.rs
  pub mod diagnostic, event, rng, value,  // núcleo de tipos (base)
  pub mod species, resources,             // datos
  pub mod perception, actions, program, expr, compile, interp, world, sim  // capas arriba

Capas (dependencia hacia abajo solamente):
  CLI -> compile/expr/lexer -> interp/program/value -> world/sim/species/resources/actions/perception -> diagnostic/event/rng

W2
  World es dueño de terrains/objects; Simulation lee a través de world; una sola reserva por terreno/objeto.
```

## Tests

Nivel: integración y revisión. TDD: escribe primero el test que falle (reserva de terreno distinta entre World y Simulation tras daño y fin de tick), luego implementa.

- `world_tests`: dañar un terreno en `World`, hacer `end_of_tick_terrain_and_objects` en `Simulation`, verificar que `World.reserve_at` y la reserva de `Simulation` coinciden.
- `sim_tests`: fin de tick regenera reservas sobre el catálogo único.
- `cli_run`: `run` y `check` siguen funcionando igual.

## Verificación

- `cargo test -p instinct-core --test world_tests` → reservas consistentes pasan.
- `cargo test -p instinct-core --test sim_tests` → fin de tick sobre catálogo único pasa.
- `cargo test -p instinct-cli --test cli_run` → API CLI intacta.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo dan la misma historia que antes del issue.
- Medición de memoria y tiempo por tick registrada antes y después de F4 (esta issue cierra la medición de salida de fase).

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance + F4 completa (todos los issues en `done`).

## Si te bloqueas

Si `Simulation { world: World }` hace imposible unificar catálogos sin mover `World` dentro de `Simulation`, para y repórtalo: la alternativa es que `Simulation` no tenga copias propias y todo fluya por `world`. No clones catálogos ni crees dueños dobles.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`; marca todos los issues de F4 como `done`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f4-08-capas-modulos-limites (sobre develop). Depende de F4-04; cierre de F4.
Evidencia AC por AC: pendiente.
Medición de F4 (memoria y tiempo por tick, antes/después): registrar en `PHASE.md#registro-de-ejecución`.