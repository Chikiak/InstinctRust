---
id: F3-02
title: Percepciones de cercanía, más cercano estable y rugidos
phase: F3
status: done
depends_on: [F3-01]
size: L
---

# F3-02 · Percepciones de cercanía, más cercano estable y rugidos

## Objetivo

Implementar las catorce percepciones sobre lo cercano (`allies_near`, `enemies_near`,
`objects_near`, `enemy_dist`, `enemy_dx`, `enemy_dy`, `ally_dist`, `ally_dx`, `ally_dy`,
`object_dist`, `object_dx`, `object_dy`, `roars_near`, `last_roar`) sobre un único recorrido
determinista del mundo, con una regla fija de empate, y hacer que los rugidos del tick anterior
sean legibles dentro del radio salvo los propios.

## Alcance

**Haz**
- Añadir en `perception.rs` los tipos `Nearest` (`dist`, `dx`, `dy`) y `Nearby` (tres conteos, tres
      `Option<Nearest>`, `roars`, `last_roar`) y el método `nearby` en el trait `CreatureView`.
- Calcular `Nearby` en `world.rs` con un solo recorrido: criaturas en orden de identificador
      ascendente, rejilla por filas y columnas ascendentes, rugidos en el orden del vector. Entre
      varios candidatos a la misma distancia gana el primero de ese recorrido.
- Distancia de Chebyshev; dentro del radio es distancia menor o igual que `vision`; la casilla
      propia queda fuera de conteos y de "más cercanos"; sin objetivo visible, `*_dist` vale -1 y
      las direcciones 0.
- Catorce tipos de percepción nuevos en `perception.rs`, cada uno leyendo un campo de `nearby()`, y
      su registro en `PerceptionCatalog::default()` y en `Catalog::default()`.
- `Roar` guarda el identificador del emisor (`crates/instinct-core/src/world.rs`, `event.rs` no
      cambia) y `World::add_roar` lo recibe; `Roar::apply` en `actions.rs` lo pasa.
- `roars_near` cuenta rugidos de otras criaturas del tick anterior dentro del radio; `last_roar` es
      el texto del último de ellos, o texto vacío.
- Anotar en `docs/ambiguedades.md` las filas A14, A15 y A16 con las decisiones tomadas.
- Pruebas: una por grupo de percepciones contra el mundo real, el escenario de rugido un tick
      después, el determinismo con semilla fija y la lista completa de las 31 percepciones.
- `cargo test -p instinct-core`, suite completa, clippy y fmt en verde.

**No hagas**
- No implementes `see`, `name` ni las constantes (F3-03).
- No cambies el momento en que se resuelven las percepciones ni introduzcas cacheo (decisión 3 de
      `PHASE.md`).
- No alteres la rotación de rugidos de `sim.rs` ni el evento `Roared`.
- No toques la semántica de las diez percepciones de F3-01.

### Archivos previstos
- `crates/instinct-core/src/perception.rs` — modificar — `Nearest`, `Nearby`, `nearby` en
      `CreatureView`, catorce percepciones, registro por defecto.
- `crates/instinct-core/src/world.rs` — modificar — cálculo de `Nearby` en una consulta de `World`,
      implementación de `nearby`/`roars`/`last_roar` en `CreatureSnapshot`, `creature_id` en
      `Roar`, firma de `add_roar`.
- `crates/instinct-core/src/actions.rs` — modificar — `Roar::apply` pasa el identificador, registro
      de las catorce percepciones en `Catalog::default()`.
- `crates/instinct-core/src/sim.rs` — modificar — solo si la rotación de rugidos necesita ajuste
      para que el emisor quede registrado.
- `crates/instinct-core/tests/perception_tests.rs` — modificar — doble `EmptyView` ampliado y
      pruebas de cercanía contra el mundo real.
- `crates/instinct-core/tests/world_tests.rs` — modificar — pruebas del cálculo de `Nearby` y del
      empate.
- `crates/instinct-core/tests/sim_tests.rs` — modificar — rugido en el tick N leído en el N+1 y
      determinismo con semilla.
- `docs/ambiguedades.md` — modificar — filas A14, A15, A16.

## Criterios de aceptación

- [ ] **AC1** `allies_near`, `enemies_near` y `objects_near` cuentan, dentro del radio, criaturas aliadas, criaturas enemigas y objetos, excluyendo siempre la casilla propia. Verif: `test_near_counts_exclude_own_cell` | `cargo test -p instinct-core --test perception_tests`.
- [ ] **AC2** Las ocho casillas vecinas están a distancia 1; una casilla a distancia exactamente `vision` entra y una a `vision + 1` queda fuera. Verif: `test_vision_radius_is_chebyshev` | `cargo test -p instinct-core --test world_tests`.
- [ ] **AC3** `enemy_dist`, `enemy_dx`, `enemy_dy` (y las versiones de aliado y objeto) apuntan al más cercano de cada clase con las direcciones en {-1, 0, 1}; sin ninguno visible la distancia vale -1 y las direcciones 0. Verif: `test_nearest_target_reports_direction` | `cargo test -p instinct-core --test perception_tests`.
- [ ] **AC4** Con varios candidatos a la misma distancia se elige siempre el primero del recorrido fijo (criaturas por identificador ascendente, objetos por filas y columnas ascendentes) y la elección no cambia mientras la situación no cambie. Verif: `test_nearest_tie_break_is_stable` | `cargo test -p instinct-core --test world_tests`.
- [ ] **AC5** `roars_near` cuenta los rugidos recibidos de otras criaturas dentro del radio y `last_roar` devuelve el texto del último de ellos; sin ninguno, el conteo es 0 y el texto vacío. Verif: `test_roars_near_and_last_roar` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC6** Un rugido emitido en el tick N es legible en el N+1 dentro del radio y deja de serlo en el N+2; una criatura no cuenta ni lee su propio rugido. Verif: `test_roar_is_heard_one_tick_only` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC7** Las catorce percepciones resuelven por catálogo en los dos catálogos por defecto, y la lista completa de las 31 percepciones del enunciado resuelve sin cadena de `if` por nombre. Verif: `test_default_catalogs_resolve_all_31_perceptions` | `cargo test -p instinct-core --test perception_tests`.
- [ ] **AC8** Una criatura que usa las percepciones de enemigo y de objeto corre en la simulación: dos corridas con la misma semilla producen la misma historia. Verif: `test_near_perceptions_are_deterministic` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC9** Las decisiones de ambigüedad quedan anotadas en `docs/ambiguedades.md` (A14 rugidos propios excluidos y "último" como último emitido; A15 empate del más cercano; A16 casilla propia fuera de los conteos). Verif: lectura de `docs/ambiguedades.md`.
- [ ] **AC10** Suite completa sin regresiones y puertas de calidad en verde. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe** (F3-01): el trait `CreatureView` con los diez accesores de sí misma y mundo, sus
diez percepciones registradas en los dos catálogos por defecto, y las consultas de `World` para
terreno y totales. `Roar` ya se almacena en `roars_current`/`roars_previous` con
`previous_roars`/`rotate_roars` (F1-04) y la rotación ocurre al final de cada tick en
`crates/instinct-core/src/sim.rs`.

**Contratos que debes respetar**: los de `PHASE.md#contratos-compartidos` — un solo recorrido con
orden fijo, distancia de Chebyshev, casilla propia excluida, `nearby()` recalcula en cada lectura,
registro doble en los catálogos por defecto.

**Ojo**: los objetos solo pueden estar en casillas sin criatura, así que la casilla propia nunca
tiene objeto; aun así, el recorrido excluye la propia casilla de todos los conteos por simetría.
Los rugidos guardan la posición del emisor en el momento de rugir: la distancia se mide desde la
casilla actual de quien escucha. El `id` del emisor es la única forma fiable de excluir el rugido
propio, porque la especie y la posición pueden coincidir con las de otra criatura.

### Leer primero
- `crates/instinct-core/src/perception.rs` — el trait `CreatureView` ampliado en F3-01 y un tipo de
      percepción completo, para imitar el patrón de los catorce nuevos.
- `crates/instinct-core/src/world.rs` — `CreatureSnapshot`, `creatures`, `content`, `terrains`,
      `objects`, `roars_previous` y `add_roar`: todo lo que recorre el cálculo.
- `crates/instinct-core/src/sim.rs` — `step_tick` y `rotate_roars`, para ver cuándo los rugidos
      pasan a ser "del tick anterior".
- `crates/instinct-core/src/actions.rs` — `Roar::apply` y `Catalog::default()`.
- `crates/instinct-core/tests/perception_tests.rs` — el ayudante `empty_world` y el doble
      `EmptyView` ampliado por F3-01.

## Especificación

Recorrido único, en este orden: primero las criaturas vivas en orden de identificador ascendente
(excluida la propia), clasificándolas en aliada o enemiga por igualdad de facción y quedándose con
la primera estrictamente más cercana de cada clase; después la rejilla por filas ascendentes y, en
cada fila, por columnas ascendentes, para objetos; por último los rugidos del tick anterior en el
orden del vector. Solo entran las casillas con distancia de Chebyshev menor o igual que `vision`
desde la casilla de la criatura.

`Nearby` reúne: conteos de aliados, enemigos y objetos; para cada clase, el `Nearest` con
`dist` (distancia de Chebyshev), `dx` y `dy` normalizados a {-1, 0, 1}; `roars` (rugidos ajenos
dentro del radio) y `last_roar` (texto del último de esos rugidos). Las catorce percepciones leen
un campo cada una; ninguna recorre el mundo por su cuenta.

Cada criatura percibe su propio radio: dos criaturas en la misma casilla no son posibles, y dos a
distancia 1 se ven mutuamente si su `vision` es al menos 1.

### Pasos sugeridos (opcional)
1. Escribe primero las pruebas del cálculo de `Nearby` (conteos, empate, radio) y el doble
   `EmptyView` ampliado; deben fallar por métodos que no existen.
2. Implementa el cálculo en `World` y el método `nearby` de la instantánea.
3. Añade las catorce percepciones y regístralas en los dos catálogos por defecto.
4. Añade el identificador del emisor a `Roar` y cierra con las pruebas de rugido, determinismo y la
   lista de las 31 percepciones.

## Tests

Nivel: unitario en `crates/instinct-core/tests/world_tests.rs` (cálculo de `Nearby`, empates,
límite del radio) e integración en `perception_tests.rs` (las catorce percepciones contra el mundo
real) y `sim_tests.rs` (rugido un tick después, determinismo). Nada de contextos falsos salvo
`EmptyView`. Determinismo: semilla fija y colocaciones fijas en los escenarios de simulación.

## Verificación
- `cargo test -p instinct-core --test world_tests` → conteos, radio y empate pasan.
- `cargo test -p instinct-core --test perception_tests` → las catorce percepciones y la lista de 31
      pasan.
- `cargo test -p instinct-core --test sim_tests` → rugidos y determinismo pasan.
- `cargo test --workspace` → sin regresiones de F0, F1, F2 y F3-01.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin
stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si excluir el rugido propio exige cambiar el evento `Roared` o la rotación de `sim.rs` de forma que
algo existente deje de compilar, para y repórtalo. Decisiones que NO puedes tomar tú: cambiar la
regla de empate, cachear `Nearby`, contar la casilla propia, o hacer que un rugido se oiga más de
un tick.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: feat/f3-02-percepciones-cercania (sobre develop). Commit pendiente.

Evidencia AC por AC:
- AC1 PASS: `test_near_counts_exclude_own_cell` en `tests/perception_tests.rs` — `allies_near`/`enemies_near`/`objects_near` excluyen la casilla propia. `cargo test -p instinct-core --test perception_tests` verde.
- AC2 PASS: `test_vision_radius_is_chebyshev` en `tests/world_tests.rs` — 8 vecinas a distancia 1, casilla a distancia == vision entra, vision+1 queda fuera. `cargo test -p instinct-core --test world_tests` verde.
- AC3 PASS: `test_nearest_target_reports_direction` y `test_nearest_target_none_visible` en `tests/perception_tests.rs` — `*_dist` devuelve -1 y direcciones 0 cuando no hay objetivo; dx/dy en {-1,0,1} cuando lo hay. `cargo test -p instinct-core --test perception_tests` verde.
- AC4 PASS: `test_nearest_tie_break_is_stable` en `tests/world_tests.rs` — empate resuelto por primer candidato del recorrido fijo; estable entre lecturas. `cargo test -p instinct-core --test world_tests` verde.
- AC5 PASS: `test_roars_near_and_last_roar` en `tests/sim_tests.rs` — `roars_near` cuenta rugidos ajenos, `last_roar` devuelve texto del último. `cargo test -p instinct-core --test sim_tests` verde.
- AC6 PASS: `test_roar_is_heard_one_tick_only` en `tests/sim_tests.rs` — rugido en tick N se lee en N+1, no en N+2; propio excluido por `creature_id`. `cargo test -p instinct-core --test sim_tests` verde.
- AC7 PASS: `test_default_catalogs_resolve_all_31_perceptions` en `tests/perception_tests.rs` — las 31 percepciones resuelven en ambos catálogos por defecto. `cargo test -p instinct-core --test perception_tests` verde.
- AC8 PASS: `test_near_perceptions_are_deterministic` en `tests/sim_tests.rs` — misma semilla, misma historia de dichos. `cargo test -p instinct-core --test sim_tests` verde.
- AC9 PASS: filas A14, A15, A16 anotadas en `docs/ambiguedades.md`.
- AC10 PASS: `cargo test --workspace` (284 tests ok), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), `cargo fmt --all --check` (exit 0).

Cambios de firma autorizados aplicados (según PHASE.md#cambios-de-firma):
- `CreatureView`: añade `nearby(&self) -> Nearby`. Propagado a `FixedView` (pruebas de src/perception.rs), `EmptyView` (tests/perception_tests.rs) y `CreatureSnapshot` (world.rs).
- `Roar` y `add_roar`: añaden `creature_id: CreatureId`. Propagado a `Roar::apply` (actions.rs) y a la prueba `test_roar_rotates_to_previous_tick` (tests/sim_tests.rs).
- No se tocó `expr::parse` ni `EvalContext` (quedan para F3-03). `World::set_variable` no se cambió (funciones y constantes no existen aún; F3-03 las añade).

Nuevos tipos en `perception.rs`: `Nearest` (`dist`, `dx`, `dy`), `Nearby` (`allies`, `enemies`, `objects`, `ally`, `enemy`, `object`, `roars`, `last_roar`), y 14 percepciones (`AlliesNear`, `EnemiesNear`, `ObjectsNear`, `EnemyDist`, `EnemyDx`, `EnemyDy`, `AllyDist`, `AllyDx`, `AllyDy`, `ObjectDist`, `ObjectDx`, `ObjectDy`, `RoarsNear`, `LastRoar`). Registro doble en `PerceptionCatalog::default()` y `Catalog::default()`.

Cálculo de `Nearby` en `World::nearby()` (world.rs): un solo recorrido — criaturas por id ascendente, rejilla por filas/columnas ascendentes para objetos, rugidos de `roars_previous` en orden del vector. Distancia de Chebyshev; radio <= vision; casilla propia excluida; `dx`/`dy` normalizados con `signum()`.

Pruebas existentes ajustadas: `EmptyView` en `tests/perception_tests.rs` implementa `nearby()`; `add_roar` llamada en `tests/sim_tests.rs` pasa `creature_id`. Ninguna aserción suavizada.

Desviaciones: ninguna. Todo dentro del alcance de F3-02.

Hechos para F3-03: `nearby` y `Nearby`/`Nearest` están listos para que `see`/`name` reutilicen el radio y las reglas de empate. `Roar` ya tiene `creature_id`.
