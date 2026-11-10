---
id: F4-05
title: Flyweight para especies (y catálogos compartidos)
phase: F4
status: todo
depends_on: [F4-04]
size: M
---

# F4-05 · Flyweight para especies (y catálogos compartidos)

## Objetivo

Aplicar el patrón Flyweight (sección 4.2 de `docs/PATRONES_DISENO_ARQUITECTURA.md`) para que las especies, terrenos y objetos se compartan en lugar de clonarse por criatura/casilla. `Creature.species` pasa de `Species` (clonado por criatura) a una referencia compartida (`Rc<Species>` o `SpeciesId` con el catálogo como dueño). Depende de F4-04 porque los almacenes indexados facilitan acceder a la especie compartida sin duplicar datos.

## Alcance

**Haz**
- `Creature.species` deja de ser `Species` (valor clonado) y pasa a ser `Rc<Species>` (compartido, inmutable) o `SpeciesId` resuelto contra el catálogo del mundo. Elegir `Rc<Species>` si el programa de la especie se comparte por referencia; documentar la elección.
- `Species` queda como dato intrínseco inmutable compartido: `name`, `faction`, `health`, `vision`, `lifespan`, `program: Option<Rc<Program>>`. El estado extrínseco (health, age, posición, variables, pc) sigue en `Creature`.
- `World` y `Simulation` mantienen un catálogo de especies dueño de las `Rc`; las criaturas apuntan con `Rc<Species>` (clonar `Rc` es barato, no clona `Species`).
- `resources.rs` (`load_creatures_folder`) devuelve especies dueñas; `World::from_map` / `place_creature` las envuelve en `Rc`.
- Mismo comportamiento: misma semilla, misma historia, misma salida de `draw`.
- Pruebas: dos criaturas de la misma especie comparten el mismo `Rc<Species>` (mismo puntero); modificar la especie original no afecta a las criaturas (inmutabilidad); `draw` muestra el nombre de especie correctamente.

**No hagas**
- No mutes `Species` después de compartirla (rompería Flyweight). Si algo de la especie muta, sepáralo como estado extrínseco en `Creature`.
- No cambies `Creature.id`, `x`, `y`, `health`, `age`, `lifespan`, `variables`, `pc`.
- No introduzcas `unwrap`/`expect`/`panic!` sobre datos de entrada.
- No añadas dependencias externas (`Rc` de `std` está permitido).

### Archivos previstos
- `crates/instinct-core/src/species.rs` — modificar — `Species` inmutable compartible; `program: Option<Rc<Program>>`.
- `crates/instinct-core/src/world.rs` — modificar — `Creature.species: Rc<Species>`, `World` dueño de especies, `place_creature` envuelve en `Rc`.
- `crates/instinct-core/src/resources.rs` — modificar — `load_creatures_folder` devuelve especies dueñas del catálogo.
- `crates/instinct-core/src/sim.rs` — verificar que no clona especies innecesariamente.
- `crates/instinct-core/tests/world_tests.rs` — modificar — compartir de especie entre criaturas.
- `crates/instinct-core/tests/species_tests.rs` — modificar — inmutabilidad y `Rc` compartido.
- `crates/instinct-core/tests/annex_examples_tests.rs` — modificar — no-regresión de semilla.

## Criterios de aceptación

- [ ] **AC1** `Creature.species` es `Rc<Species>` (o `SpeciesId` + catálogo) y dos criaturas de la misma especie comparten la misma instancia. Verif: `test_creatures_share_species_rc` | `cargo test -p instinct-core --test world_tests`.
- [ ] **AC2** `Species` es inmutable tras el registro; `program` es `Option<Rc<Program>>`. Verif: `test_species_is_immutable_shared` | `cargo test -p instinct-core --test species_tests`.
- [ ] **AC3** `place_creature` envuelve la especie del catálogo en `Rc` y no clona el `Species` completo. Verif: `cargo test --workspace`.
- [ ] **AC4** Determinismo: misma semilla, misma historia (misma salida de `draw` y eventos) antes y después del issue. Verif: dos corridas `--seed 7`.
- [ ] **AC5** Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Suite completa + clippy + fmt en verde.

## Contexto

**Ya existe**: `Species { name, faction, health, vision, lifespan, program: Option<Rc<Program>> }` en `species.rs`; `Creature.species: Species` (clonado por criatura) en `world.rs`; `load_creatures_folder` en `resources.rs`. W5 (ECS) ya se aplica en F4-04. La sección 4.2 del documento sugiere `Rc<Species>` o `SpeciesId`.

**Contratos que debes respetar**: `Creature.species` sigue resolviendo nombre, facción, salud base, visión y longevidad; `draw` imprime el nombre de la especie; cero pánicos en el núcleo.

**Ojo**: si eliges `SpeciesId` en lugar de `Rc<Species>`, `World` necesita un método `species(id) -> &Species` y todas las consultas que leen la especie pasan por el catálogo. `Rc<Species>` es más simple para esta fase; documenta la elección en el cierre.

### Leer primero
- `crates/instinct-core/src/species.rs` — `Species`, `new`, `with_program`.
- `crates/instinct-core/src/world.rs` — `Creature.species`, `place_creature`, `World::from_map`.
- `crates/instinct-core/src/resources.rs` — `load_creatures_folder`, `LoadedResources.creatures`.
- `crates/instinct-core/tests/species_tests.rs` — pruebas existentes de especies.

## Especificación

```
species.rs
  Species { name, faction, health, vision, lifespan, program: Option<Rc<Program>> }
  // inmutable tras crear; no hay setters

world.rs
  Creature { species: Rc<Species>, ... }  // ya no clona Species por criatura
  World { especies_duenas: Vec<Rc<Species>> (o las guarda en el catálogo) }
  place_creature(especie_del_catalogo, ...) -> Creature { species: Rc::clone(&especie), ... }

resources.rs
  load_creatures_folder -> Vec<Species> (dueño); World las envuelve en Rc
```

## Tests

Nivel: unitario e integración. TDD: escribe primero el test que falle (dos criaturas de la misma especie clonarían el `Species` entero), luego implementa.

- `world_tests`: dos `place_creature` de "Grass"/misma especie → `Rc::ptr_eq(c1.species, c2.species)` es true.
- `species_tests`: `Species` sin setters públicos; `program` compartido por `Rc`.
- `annex_examples_tests`: `--seed 7` sobre los cuatro ejemplos → misma historia que la versión base.

## Verificación

- `cargo test -p instinct-core --test world_tests` → compartido de especie pasa.
- `cargo test -p instinct-core --test species_tests` → inmutabilidad y `Rc` pasan.
- `cargo test -p instinct-core --test annex_examples_tests` → no-regresión de semilla 7.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo dan la misma historia que antes del issue.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si `Rc<Species>` obliga a cambiar muchas firmas que reciben `&Species`, para y repórtalo: considera `SpeciesId` + `World::species(id) -> &Species` como alternativa. No clones `Species` por criatura ni uses `Rc<RefCell<Species>>`.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f4-05-flyweight-especies (sobre develop). Depende de F4-04.
Evidencia AC por AC: pendiente.
Cambios de firma autorizados: `Creature.species` de `Species` a `Rc<Species>`.