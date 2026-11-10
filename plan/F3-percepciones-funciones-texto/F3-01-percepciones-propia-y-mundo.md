---
id: F3-01
title: Percepciones sobre sí misma y sobre el mundo
phase: F3
status: todo
depends_on: []
size: M
---

# F3-01 · Percepciones sobre sí misma y sobre el mundo

## Objetivo

Completar las diez percepciones que describen a la criatura y al mundo (`vision`, `species`,
`faction`, `terrain_here`, `reserve_here`, `width`, `height`, `total_creatures`, `total_allies`,
`total_enemies`) ampliando el contrato `CreatureView` y delegando en consultas nuevas del `World`.
Con ellas, criaturas como el Hobbit o el Ent ya pueden decidir con `reserve_here` o
`total_allies`.

## Alcance

**Haz**
- Ampliar el trait `CreatureView` de `crates/instinct-core/src/perception.rs` con los diez
  accesores: `vision`, `species_name`, `faction_name`, `terrain_here`, `reserve_here`, `width`,
  `height`, `total_creatures`, `total_allies`, `total_enemies`.
- Implementarlos en `CreatureSnapshot` (`crates/instinct-core/src/world.rs`) delegando en consultas
  nuevas de `World`; la instantánea no guarda estado propio ni cachea nada.
- Añadir en `World` las consultas que faltan: terreno (nombre y reserva) de una casilla, dimensiones
  del mapa y recuento de criaturas vivas por facción excluyendo a una dada.
- Diez tipos de percepción nuevos en `perception.rs`, cada uno leyendo un único accesoro, y su
  registro en `PerceptionCatalog::default()` y en `Catalog::default()` (los dos catálogos por
  defecto).
- Pruebas: una por percepción nueva contra el mundo real, más un escenario de simulación y la
  semántica de texto protegida con pruebas nuevas.
- `cargo test -p instinct-core`, suite completa, clippy y fmt en verde.

**No hagas**
- No implementes las percepciones de cercanía (`*_near`, `*_dist`, `*_dx`, `*_dy`, `roars_near`,
  `last_roar`): es F3-02.
- No implementes `see`, `name` ni las constantes: es F3-03.
- No cambies la semántica de `random`, `tick` ni de las percepciones ya existentes.
- No introduzcas cacheo ni refresco en bloque de percepciones al empezar el turno (decisión 3 de
  `PHASE.md`).
- No cambies la firma de `EvalContext` ni añadas el nodo `Call`: es F3-03.

### Archivos previstos
- `crates/instinct-core/src/perception.rs` — modificar — diez accesores en `CreatureView`, diez
  tipos de percepción, registro en `PerceptionCatalog::default()`.
- `crates/instinct-core/src/world.rs` — modificar — implementación de los diez accesores en
  `CreatureSnapshot` y consultas nuevas de `World`.
- `crates/instinct-core/src/actions.rs` — modificar — registro de las diez percepciones en
  `Catalog::default()`.
- `crates/instinct-core/tests/perception_tests.rs` — modificar — doble `EmptyView` ampliado y
  pruebas de las diez percepciones contra el mundo real.
- `crates/instinct-core/tests/sim_tests.rs` — modificar — escenario de simulación y semántica de
  texto de punta a punta.
- `crates/instinct-core/tests/compile_tests.rs` — modificar — asignación a los nombres nuevos
  sigue siendo error de compilación.

## Criterios de aceptación

- [ ] **AC1** El trait `CreatureView` expone los diez accesores nuevos y `CreatureSnapshot` los implementa delegando en `World`; los dobles de prueba existentes (`FixedView` en las pruebas de `src/perception.rs` y `EmptyView` en `tests/perception_tests.rs`) los implementan también sin cambiar lo que afirman. Verif: `cargo test --workspace`.
- [ ] **AC2** `vision` devuelve el atributo de cabecera de la especie; `species` y `faction` devuelven texto con el nombre de la especie y de la facción. Verif: `test_self_perceptions_report_species_data` | `cargo test -p instinct-core --test perception_tests`.
- [ ] **AC3** `terrain_here` devuelve el nombre del terreno bajo la criatura y `reserve_here` su reserva actual, que baja cuando ese terreno recibe daño y se regenera en el fin de tick. Verif: `test_terrain_here_tracks_reserve` | `cargo test -p instinct-core --test perception_tests`.
- [ ] **AC4** `width` y `height` devuelven las dimensiones del mapa; con criaturas de varias facciones colocadas, `total_creatures` cuenta todas salvo la propia y `total_allies`/`total_enemies` separan por facción. Verif: `test_totals_count_world_excluding_self` | `cargo test -p instinct-core --test perception_tests`.
- [ ] **AC5** Los diez nombres nuevos resuelven por catálogo en `Catalog::default()` y en `PerceptionCatalog::default()`, sin cadena de `if` por nombre. Verif: `test_default_catalogs_resolve_self_and_world_perceptions` | `cargo test -p instinct-core --test perception_tests`.
- [ ] **AC6** Una criatura compilada que lee `species`, `faction` y `total_enemies` corre en la simulación y emite el evento `Said` con el texto correcto. Verif: `test_simulation_reads_self_and_world_perceptions` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC7** La semántica de texto se mantiene de punta a punta: comparar dos textos da 1 o 0, mezclar texto y entero en la aritmética, en una comparación de orden o en un argumento numérico de acción es error de ejecución que mata a la criatura con su línea, y `say`/`roar` muestran un número como texto. Verif: `test_text_semantics_kill_on_type_error` | `cargo test -p instinct-core --test sim_tests`.
- [ ] **AC8** Asignar a cualquiera de los diez nombres nuevos es error de compilación. Verif: `test_compile_rejects_assignment_to_new_perceptions` | `cargo test -p instinct-core --test compile_tests`.
- [ ] **AC9** Suite completa sin regresiones y puertas de calidad en verde. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: F2 cerrada con `Diagnostic` único, cero `unwrap`/`expect` sobre datos de entrada,
catálogos con un solo dueño (`World`), `pc` y límite de líneas corregidos, aritmética comprobada en
fin de tick. `CreatureView` con siete accesores en `crates/instinct-core/src/perception.rs`;
`CreatureSnapshot` y `CreatureContext` en `crates/instinct-core/src/world.rs`; `Catalog` con
`register_perception` y `Catalog::default()` en `crates/instinct-core/src/actions.rs`; dobles
`FixedView` (pruebas de `src/perception.rs`) y `EmptyView` (`tests/perception_tests.rs`).

**Contratos que debes respetar**: los de `PHASE.md#contratos-compartidos` — `CreatureView` crece
sin cachear; un accesoro por percepción; registro doble en los dos catálogos por defecto; orden de
resolución de nombres percepción → constante → variable.

**Ojo**: la reserva del terreno es del catálogo, no de la casilla: dos casillas con el mismo
terreno comparten reserva y `reserve_here` la lee del catálogo del mundo. `total_*` cuenta
criaturas vivas, y todas las del mundo lo están porque las muertas se retiran al morir. `width`,
`height`, `total_*` y `reserve_here` necesitan convertir a `i64` sin envoltura: usa la conversión
con vuelta a comprobar que ya existe en el repo para coordenadas.

### Leer primero
- `crates/instinct-core/src/perception.rs` — el trait `Perception`, un tipo de percepción completo
      (`Health`) y `PerceptionCatalog::default()`, que es el patrón a imitar.
- `crates/instinct-core/src/world.rs` — `CreatureSnapshot`, `CreatureContext::resolve` y las
      consultas existentes de `World` (`creature`, `terrain`, `content`, `terrains`).
- `crates/instinct-core/src/actions.rs` — `Catalog::default()`, donde se registran las percepciones
      junto a las acciones, y `resolve_perception`.
- `crates/instinct-core/tests/perception_tests.rs` — el ayudante `empty_world`, el doble
      `EmptyView` y cómo se evalúa una expresión contra el mundo real.
- `crates/instinct-core/src/actions.rs` — `index_from_i64` para la conversión segura a índice.

## Especificación

| Percepción | Tipo | Origen del dato |
|---|---|---|
| `vision` | entero | atributo `vision` de la especie de la criatura |
| `species` | texto | nombre de la especie |
| `faction` | texto | facción de la especie |
| `terrain_here` | texto | nombre del terreno de la casilla de la criatura |
| `reserve_here` | entero | reserva actual de ese terreno en el catálogo del mundo |
| `width` | entero | columnas del mapa |
| `height` | entero | filas del mapa |
| `total_creatures` | entero | criaturas vivas en el mapa menos la propia |
| `total_allies` | entero | vivas de la misma facción, sin contarse |
| `total_enemies` | entero | vivas de otra facción |

`reserve_here` refleja el daño recibido por ese terreno y su regeneración de fin de tick, porque
lee la reserva del catálogo que `World` muta. Los totales se calculan sobre el estado actual del
mundo en el momento de la lectura.

### Pasos sugeridos (opcional)
1. Escribe primero las pruebas de las diez percepciones contra el mundo real y el doble `EmptyView`
   ampliado; deben fallar por métodos que no existen.
2. Amplía `CreatureView` e implementa los accesores en `CreatureSnapshot` con las consultas nuevas
   de `World`.
3. Añade los diez tipos de percepción y regístralos en los dos catálogos por defecto.
4. Cierra con el escenario de simulación y las pruebas de semántica de texto.

## Tests

Nivel: unitario e integración en `crates/instinct-core/tests/perception_tests.rs` y
`sim_tests.rs`, contra el `World` real y el catálogo real (nada de contextos falsos salvo el doble
`EmptyView`, que solo existe para probar el trait). Casos como comportamientos: cada percepción
nueva con su valor esperado, `reserve_here` antes y después de dañar el terreno, totales con dos
especies de facciones distintas, texto comparado y texto donde se espera número. Determinismo: sin
dependencia de `random` salvo en las pruebas que ya lo usan.

## Verificación
- `cargo test -p instinct-core --test perception_tests` → las diez percepciones nuevas pasan.
- `cargo test -p instinct-core --test sim_tests` → el escenario de simulación y la semántica de
      texto pasan.
- `cargo test -p instinct-core --test compile_tests` → la asignación a los nombres nuevos sigue
      rechazándose.
- `cargo test --workspace` → sin regresiones de F0, F1 y F2.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin
stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si algún accesoro no se puede implementar sin mutar el mundo o sin un `RefCell` nuevo, para y
repórtalo en "Notas de implementación": el contrato dice que la vista es de sólo lectura. Decisiones
que NO puedes tomar tú: cambiar el orden de resolución de nombres, cachear percepciones, mover el
cálculo de totales fuera de `World`, o tocar la semántica de las siete percepciones existentes.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
<!-- Lo rellena el implementador: desviaciones, hechos descubiertos que afectan a issues siguientes,
nombres reales creados (para que F3-02 los cite), deuda consciente. -->
