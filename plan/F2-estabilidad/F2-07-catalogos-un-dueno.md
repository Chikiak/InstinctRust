---
id: F2-07
title: Catálogos de terrenos y objetos con un solo dueño
phase: F2
status: done
depends_on: [F2-03, F2-06]
size: L
---

# F2-07 · Catálogos de terrenos y objetos con un solo dueño

## Objetivo

Cerrar el problema W2: hoy `World` clona `terrains` y `objects` y `Simulation` guarda copias
propias, de modo que el daño muta una copia y la regeneración de fin de tick escribe sobre la otra,
dejando las reservas desincronizadas. Al terminar, hay un único dueño de los catálogos y una sola
reserva por terreno y por objeto.

## Alcance

**Haz**

- Decidir y aplicar el dueño único. La decisión de la fase, en `PHASE.md`, es que **`World` es el
  dueño**: `Simulation` consume los catálogos al construir el mundo y deja de almacenar copias.
  Consecuencias mínimas: `Simulation::new` ya no necesita catálogos propios que divergir;
  `end_of_tick_terrain_and_objects` opera sobre los catálogos del mundo; `damage_cell_content`
  sigue mutando la misma reserva que después se regenera.
- Mantener la trayectoria de construcción para mundos a mano y pruebas: quien construye un `World`
  sigue pasando terrenos y objetos, y el mundo se queda con ellos.
- Añadir una prueba que fije la consistencia: dañar el terreno de una casilla, avanzar un tick, y
  comprobar que la regeneración parte de la reserva dañada y no de una copia intacta.
- Revisar todos los sitios que hoy dependen de las copias de `Simulation` (entre ellos la CLI, que
  le pasa `sim.terrains` y `sim.objects` a `draw`) y dejarlos coherentes con el dueño único.
- `cargo test -p instinct-core`, clippy y fmt en verde.

**No hagas**

- No cambies la ley de regeneración ni la de objetos agotados.
- No introduzcas `Rc`/`RefCell` para compartir catálogos: el dueño único basta.
- No cambies el formato del mapa ASCII ni el contenido de los ficheros de recursos.
- No toques la validación de argumentos de la CLI (F2-08) salvo para adaptar la llamada.

### Archivos previstos

- `crates/instinct-core/src/world.rs` — modificar — dueño de los catálogos, fin de tick sobre ellos.
- `crates/instinct-core/src/sim.rs` — modificar — sin copias propias de catálogos.
- `crates/instinct-cli/src/main.rs` — modificar — adaptar a la nueva fuente de catálogos.
- `crates/instinct-core/tests/sim_tests.rs`, `crates/instinct-core/tests/world_tests.rs` — modificar — prueba de consistencia.

## Criterios de aceptación

- [ ] **AC1** Existe un único dueño de `terrains` y `objects` durante la simulación; `Simulation` no mantiene una segunda copia que pueda divergir. Verif: inspección de `Simulation` + `cargo test -p instinct-core --test sim_tests` | una sola fuente de verdad y pruebas en verde.
- [ ] **AC2** El daño a una reserva y su regeneración de fin de tick leen y escriben la misma reserva. Verif: `test_terrain_damage_then_regen_consistent` | `cargo test -p instinct-core --test sim_tests` | prueba pasa.
- [ ] **AC3** Un objeto dañado hasta 0 o menos sigue desapareciendo al final del tick y libera su casilla. Verif: `test_object_exhausted_still_removed` | `cargo test -p instinct-core --test sim_tests` | prueba pasa.
- [ ] **AC4** `draw` sigue produciendo el mismo ASCII con el dueño único. Verif: `cargo test -p instinct-core --test world_tests` + escenario E2E de F1 | misma salida.
- [ ] **AC5** La CLI compila y corre con el nuevo origen de los catálogos. Verif: `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` | exit 0, mismo mapa que en F1.
- [ ] **AC6** Determinismo intacto: dos corridas con la misma semilla dan la misma historia. Verif: `test_same_seed_same_history` | `cargo test -p instinct-core --test sim_tests` | prueba pasa.
- [ ] **AC7** Suite completa sin regresiones: clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `World` con los campos `terrains` y `objects` clonados en `from_map`
(líneas 77-79 y 122-123 de `world.rs`) y en el constructor de mundo a mano (líneas 181-182);
`Simulation::new(world, terrains, objects)` con copias propias (línea 27 de `sim.rs`);
`end_of_tick_terrain_and_objects(&mut self, terrains, objects)` que regenera sobre las copias que
recibe (líneas 534-537 de `world.rs`); `strike_cell` y `damage_cell_content`, que mutan las copias
del mundo; la CLI, que llama `sim.world().draw(&sim.terrains, &sim.objects)`.
**Contratos que debes respetar**: el mundo escribe y el intérprete lee; la rejilla guarda
identificadores estables; `World` sigue siendo construible a mano desde `Map` más catálogos; las
leyes de fin de tick no cambian.
**Ojo**: con F2-03, `draw` ya devuelve `Result<String, Diagnostic>`; este issue debe dejar su firma
coherente con el dueño único (si `draw` pasa a no necesitar catálogos, adelante: es la consecuencia
natural). Cambia una sola vez y cítalo en las notas.

### Leer primero

- `crates/instinct-core/src/world.rs` — campos, constructores, `damage_cell_content`, `end_of_tick_*`, `draw`.
- `crates/instinct-core/src/sim.rs` — `Simulation`, `step_tick`, `end_of_tick_terrain_and_objects` en uso.
- `crates/instinct-cli/src/main.rs` — dónde se consumen los catálogos.
- `crates/instinct-core/tests/sim_tests.rs` y `world_tests.rs` — pruebas que afirman reservas.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` problema W2 y `docs/PATRONES_DISENO_ARQUITECTURA.md` sección 1.2.

## Especificación

Un único almacén de terrenos y objetos, propiedad del mundo. La simulación lo usa para regenerar y
para dibujar; el daño lo muta. Ninguna función recibe por parámetro una copia que pueda divergir de
la del mundo. La consistencia se demuestra con una prueba que encadena daño y regeneración sobre el
mismo terreno.

## Pasos sugeridos (opcional)

1. Escribe la prueba de consistencia (daño → tick → regeneración) y comprueba que falla con las copias actuales.
2. Mueve la propiedad de los catálogos a un único sitio y ajusta constructores.
3. Ajusta `end_of_tick_terrain_and_objects` para operar sobre el dueño único.
4. Ajusta la CLI y las pruebas afectadas.
5. `cargo test --workspace` y el gate completo.

## Tests

Nivel: unitario e integración en `sim_tests.rs`, `world_tests.rs` y los E2E de CLI. Casos como
comportamientos: daño y regeneración sobre la misma reserva, objeto agotado retirado, mismo ASCII,
misma semilla misma historia. Qué NO mockear: `World` y `Simulation` reales. Determinismo: semilla
fija. Datos: mapas y recursos de `examples/`.

## Verificación

- `cargo test -p instinct-core --test sim_tests` → pasan.
- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si el préstamo de catálogos choca con el borrow checker, la solución es el dueño único, no
`Rc<RefCell>`: si aun así hace falta compartir, para y repórtalo. Decisiones que NO puedes tomar
tú: cambiar las leyes de regeneración, introducir dependencias externas, o aplazar la prueba de
consistencia.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Dueño único: `World` es el único dueño de `terrains` y `objects` (clonados en
`World::from_map`, que conserva su firma). `Simulation` ya no almacena copias;
su campo es solo `pub world: World`.

Firmas finales:
- `pub fn World::draw(&self) -> Result<String, Diagnostic>` (valida contra
  `self.terrains` / `self.objects`; ya no recibe catálogos).
- `pub fn World::end_of_tick_terrain_and_objects(&mut self) -> Vec<Pos>` (opera
  sobre los catálogos del mundo: misma reserva que `damage_cell_content` muta).
- `pub fn Simulation::new(world: World) -> Self`.

Sitios cambiados:
- `crates/instinct-core/src/world.rs` — `draw` y `end_of_tick_terrain_and_objects` sin parámetros de catálogo.
- `crates/instinct-core/src/sim.rs` — `Simulation` sin campos `terrains`/`objects`; `Simulation::new(world)`; `step_tick` llama `self.world.end_of_tick_terrain_and_objects()`.
- `crates/instinct-cli/src/main.rs` — `Simulation::new(world)` y `sim.world().draw()` (2 sitios); la CLI deja de pasar catálogos a `draw`.

Evidencia por AC:
- AC1 (único dueño): inspección de `Simulation` (solo `world`); `cargo test -p instinct-core --test sim_tests` → 8/8 ok.
- AC2 (daño y regeneración sobre la misma reserva): `test_terrain_damage_then_regen_consistent` pasa (100 → daño 30 → 70 → tick 72 → daño 30 → 42 → tick 44).
- AC3 (objeto agotado retirado y casilla liberada): `test_object_exhausted_still_removed` pasa (reserve 100 → daño 100 → 0 → tick casilla `Empty`).
- AC4 (mismo ASCII): `cargo test -p instinct-core --test world_tests` → 12/12 ok (`test_draw_example_map`, `test_render_shows_creature`).
- AC5 (CLI): `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` → exit 0, mismo mapa que F1 (Walker muere por vida en tick 9).
- AC6 (determinismo): `test_same_seed_same_history` pasa.
- AC7 (suite): `cargo test --workspace` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo fmt --all --check` exit 0.

Pruebas existentes ajustadas por el cambio de contrato:
- `sim_tests.rs`: `Simulation::new(world)` en 6 sitios, acceso a reservas vía `sim.world.terrains`/`sim.world.objects`, `draw()` sin args, `two_walker_world()` devuelve `World`.
- `world_tests.rs`: `draw()` sin args (5 sitios), `empty_world()` devuelve `World`, `test_draw_short_catalog_errors` valida índices huérfanos contra los catálogos propios del mundo.
- `cli_run.rs`: `grass_world()` devuelve `World`; se quitó import `Object` sin uso.

Sin `Rc`/`RefCell` para catálogos, sin cambios a leyes de regeneración ni de objetos agotados, sin cambios al formato de mapa ni a recursos; zero `unwrap`/`expect` nuevos sobre datos de usuario.
