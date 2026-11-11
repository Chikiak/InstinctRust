---
id: F4-04
title: Almacenes de criaturas indexados por identificador (ECS ligero)
phase: F4
status: done
depends_on: [F4-02]
size: L
---

# F4-04 · Almacenes de criaturas indexados por identificador (ECS ligero)

## Objetivo

Reemplazar el `Vec<Creature>` con búsqueda lineal por almacenes indexados por `CreatureId` (ECS ligero, sección 4.1 de `docs/PATRONES_DISENO_ARQUITECTURA.md`), resolviendo W5 (búsqueda lineal repetida y `dead_ids.contains` O(n²)). `CreatureId(usize)` sigue siendo estable y la rejilla guarda identificadores, no referencias. Depende de F4-02 porque el almacén indexado necesita el `CreatureState` modelado.

## Alcance

**Haz**
- `World.creatures` deja de ser `Vec<Creature>` con búsqueda lineal: pasa a ser un almacén indexado por `CreatureId` (por ejemplo `Vec<Option<Creature>>` con free-list de ids libres, o `Vec<Creature>` donde `Creature.id` es estable y se reusan huecos). `CreatureId` sigue siendo `usize` estable dentro de una simulación.
- `creature(id)`, `creature_mut(id)`, `creature_position(id)` pasan de O(n) a O(1); se elimina `dead_ids.contains` O(n²) de los bucles de `sim.rs`.
- La rejilla `content: Vec<Vec<CellContent>>` sigue guardando `CellContent::Creature(CreatureId)` (identificadores estables, no referencias).
- `place_creature` devuelve un `CreatureId` estable y reusa huecos libres si los hay; `remove_creature` libera el hueco sin invalidar ids de otras criaturas.
- `sim.step_tick` itera sobre los ids vivos del almacén (no sobre un `Vec` compacto con buscados por id).
- `next_creature_id` deja de ser necesario como contador puro (o se mantiene como cursor del free-list); documentar.
- End of tick, acciones y percepciones siguen funcionando igual: misma semilla, misma historia.
- Pruebas: `place_creature` devuelve ids estables; `remove_creature` libera el hueco y un `place_creature` posterior puede reusarlo; acceso O(1) por id; misma semilla = misma historia.

**No hagas**
- No cambies `CreatureId(pub usize)` ni la semántica de id estable.
- No cambies la rejilla: sigue guardando `CellContent::Creature(CreatureId)`.
- No cambies el orden de turnos, el límite de 100 líneas ni el fin de tick.
- No introduzcas `unwrap`/`expect`/`panic!` sobre datos de entrada.
- No pongas referencias a criaturas en la rejilla.

### Archivos previstos
- `crates/instinct-core/src/world.rs` — modificar — `World.creatures` como almacén indexado por `CreatureId`, `place_creature`, `remove_creature`, `creature`/`creature_mut`/`creature_position` O(1), `next_creature_id`/free-list.
- `crates/instinct-core/src/sim.rs` — modificar — iteración sobre ids vivos, sin búsquedas lineales ni `dead_ids.contains` O(n²).
- `crates/instinct-core/src/actions.rs` — modificar — uso de `creature(id)` O(1) (sin cambios de comportamiento).
- `crates/instinct-core/src/interp.rs` — modificar — si accede a criatura por id, usar consulta O(1).
- `crates/instinct-core/tests/world_tests.rs` — modificar — ids estables, reuso de huecos, acceso O(1).
- `crates/instinct-core/tests/sim_tests.rs` — modificar — mismo comportamiento con almacén indexado.
- `crates/instinct-core/tests/annex_examples_tests.rs` — modificar — no-regresión de semilla.

## Criterios de aceptación

- [ ] **AC1** `World.creatures` es un almacén indexado por `CreatureId`; `creature(id)`, `creature_mut(id)`, `creature_position(id)` son O(1) y no hay búsqueda lineal por id. Verif: `cargo test --workspace`.
- [ ] **AC2** `place_creature` devuelve `CreatureId` estable; `remove_creature(id)` libera el hueco y un `place_creature` posterior puede devolver ese mismo id (reuso) sin corromper otras criaturas. Verif: `test_place_creature_returns_stable_id_and_reuses_slot` | `cargo test -p instinct-core --test world_tests`.
- [ ] **AC3** No hay `dead_ids.contains` dentro de bucles; la iteración de `sim.step_tick` recorre ids vivos del almacén. Verif: `test_sim_iterates_live_ids_only` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC4** La rejilla sigue guardando `CellContent::Creature(CreatureId)`; `draw` no panea ante id huérfano (W1 ya cerrado en F2, se mantiene). Verif: `test_draw_no_orphan_panic` | `cargo test -p instinct-core --test world_tests`.
- [ ] **AC5** Determinismo: misma semilla, misma historia (acciones, muertes, nacimientos, fin de tick) antes y después del issue. Verif: dos corridas `--seed 7`.
- [ ] **AC6** Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Suite completa + clippy + fmt en verde.

## Contexto

**Ya existe**: `CreatureId(pub usize)`, `CellContent::Creature(CreatureId)`, `World.creatures: Vec<Creature>`, `next_creature_id: usize`, `dead_ids` (o similar) en `sim.rs`/`world.rs`, búsqueda lineal O(n) en `creature`/`creature_mut`/`creature_position` (W5 abierto). `CreatureState` ya existe (F4-02). La rejilla ya guarda IDs, no referencias.

**Contratos que debes respetar**: ids estables, rejilla con IDs, `place_creature`/`remove_creature` con `Diagnostic`, fin de tick completo, cero pánicos en el núcleo.

**Ojo**: el cambio de `Vec<Creature>` a almacén indexado toca casi todos los archivos que acceden a criaturas por id. Hazlo con la suite completa como red de seguridad; si algún sitio se rompe, para y repórtalo antes de seguir.

### Leer primero
- `crates/instinct-core/src/world.rs` — `creatures: Vec<Creature>`, `next_creature_id`, `creature`/`creature_mut`/`creature_position`, `place_creature`, `remove_creature`.
- `crates/instinct-core/src/sim.rs` — `step_tick`, iteración sobre criaturas, `dead_ids`.
- `crates/instinct-core/src/actions.rs` — cómo acceden las acciones a criaturas por id.
- `crates/instinct-core/src/interp.rs` — si usa `creature(id)`.

## Especificación

```
World.creatures: Vec<Option<Creature>>   // índice == CreatureId.0; None = hueco libre
World.free_ids: Vec<usize>               // ids liberados para reuso

place_creature(species, x, y)
  if free_ids no vacío: id = free_ids.pop(); creaturas[id] = Some(creature)
  else: id = creaturas.len(); creaturas.push(Some(creature))
  rejilla[y][x] = Creature(id)
  devuelve CreatureId(id)

remove_creature(id)
  creaturas[id] = None; free_ids.push(id); rejilla[pos] = Empty

creature(id) / creature_mut(id) / creature_position(id) → O(1) indexando

sim.step_tick
  for id in ids_vivos(almacén):  // filtrar Some y no-muertos
    run_creature_turn(id)
  end_of_tick
```

## Tests

Nivel: unitario e integración. TDD: escribe primero el test que falle (buscar criatura por id es O(n) o el id se reusa mal), luego implementa.

- `world_tests`: `place_creature` dos veces → ids 0 y 1; `remove_creature(0)`; `place_creature` otra vez → id 0 (reuso); `creature(0)` y `creature(1)` O(1); `draw` sin pánico.
- `sim_tests`: misma simulación que antes del issue → misma secuencia de eventos y mapa final.
- `annex_examples_tests`: `--seed 7` sobre los cuatro ejemplos → misma historia que la versión base.

## Verificación

- `cargo test -p instinct-core --test world_tests` → almacén indexado y reuso de ids pasan.
- `cargo test -p instinct-core --test sim_tests` → iteración sobre ids vivos pasa.
- `cargo test -p instinct-core --test annex_examples_tests` → no-regresión de semilla 7.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los cuatro ejemplos del anexo dan la misma historia que antes del issue.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si `remove_creature` en medio de `step_tick` invalida el iterador de ids vivos, para y repórtalo: recolecta primero los ids vivos en un `Vec` y itera sobre esa copia, o marca muertas y depura al final del tick. No uses referencias a `Creature` desde la rejilla.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f4-04-almacenes-criaturas-ecs (sobre develop). Depende de F4-02.
Commit: c4fa709 (feat: almacenes de criaturas indexados por CreatureId (ECS ligero) (F4-04)).

Evidencia AC por AC:
- AC1 PASS: `World.creatures: Vec<Option<Creature>>` (`world.rs`), índice == `CreatureId.0`, `None` = hueco libre; `creature`/`creature_mut`/`creature_position` indexan directo O(1); `live_creature_ids()` para `sim.step_tick`. Sin `find(|c| c.id == ...)` por id en el núcleo. `cargo test --workspace` OK.
- AC2 PASS: `place_creature` reusa `free_ids` (LIFO) o crece el almacén; `remove_creature` pone `None` y pushea al free-list sin invalidar otros ids. `test_place_creature_returns_stable_id_and_reuses_slot`: coloca ids 0,1; remueve 0; vuelve a colocar → id 0; `creature(0)`/`creature(1)` correctas. `cargo test -p instinct-core --test world_tests` OK.
- AC3 PASS: `sim.step_tick` itera `world.live_creature_ids()` (snapshot de ids vivos); `run_creature_turn` y el decremento de sueño usan `creature(id)`/`creature_mut(id)` O(1). `test_sim_iterates_live_ids_only`: hueco en el medio ignorado, solo ids vivos envejecen. `cargo test -p instinct-core --test sim_tests` OK. Cero `dead_ids.contains`.
- AC4 PASS: rejilla sigue con `CellContent::Creature(CreatureId)`; `draw` usa `creature(*id)` y falla con diagnóstico ante id huérfano sin pánico. `test_draw_no_orphan_panic`. `cargo test -p instinct-core --test world_tests` OK.
- AC5 PASS: determinismo intacto (misma semilla = misma historia): `test_annex_seed7_sequence_unchanged` (seed 7, 10 ticks, 4 ejemplos) y `cargo test --workspace` verdes; CLI: `run --root examples --map "Plain with rocks" --seed 7 --ticks 10 --place <Especie>@2,0` dos corridas por especie byte a byte iguales (fc.exe).
- AC6 PASS: cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` (exit 0, sin warnings), `cargo fmt --all --check` (exit 0) verdes.

Cambios de firma autorizados aplicados: `World.creatures` deja de ser `Vec<Creature>`; `next_creature_id` eliminado y sustituido por `free_ids: Vec<usize>` (cursor del free-list; cuando está vacío el próximo id es `creatures.len()`); `place_creature`/`remove_creature` mantienen firma con reuso de huecos. `creature`/`creature_mut` pasan a `pub(crate)` para que `actions.rs` y `sim.rs` accedan O(1).
Archivos tocados: `crates/instinct-core/src/world.rs`, `crates/instinct-core/src/sim.rs`, `crates/instinct-core/src/actions.rs`, `crates/instinct-core/src/interp.rs` (ningún cambio; solo revisado), tests `world_tests.rs`, `sim_tests.rs`, `annex_examples_tests.rs`, `actions_tests.rs`, `perception_tests.rs`, `interp_tests.rs`, `cli_run.rs`.
Comportamiento no alterado: orden de turnos, límite de 100 líneas, fin de tick, eventos y semilla igual que antes del issue.