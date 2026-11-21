---
id: F6-04
title: Percepciones, see y name
phase: F6
status: todo
depends_on: [F6-01]
size: M
file: f6-04-percepciones-see-name.md
---

# F6-04 · Percepciones, `see` y `name`

## Objetivo

Scenarios que comprueban las percepciones del enunciado 2.6 con valores
observables: las treinta y una percepciones propias, de mundo y de cercanía, las
cinco constantes de `see` —incluido el borde del mapa comportándose como objeto y
la propia casilla como aliado—, y `name` devolviendo la especie, el objeto, el
terreno o el texto vacío fuera del radio de visión. Para hacerlas observables sin
cambiar el núcleo, las criaturas del banco dicen por el registro lo que perciben.

## Alcance

**Haz**
- Los terrenos Sand y Crystal y un mapa mixto con franjas de los dos terrenos,
  para que `name` y `terrain_here` distingan materiales.
- Criaturas que dicen lo que ven y perciben: un explorador con visión amplia y
  uno con visión mínima, más compañeros aliados y enemigos quietos.
- Scenarios en `examples/README.md`: una percepción o grupo cohesionado por entrada,
  con el comando exacto y la línea de registro esperada.
- Pruebas en `crates/instinct-cli/tests/cli_bank/perceptions.rs` (líneas de
  registro y rejilla) y en `crates/instinct-core/tests/bank/perceptions.rs`
  (valores exactos de las percepciones y de `see`/`name`).

**No hagas**
- No pruebes `random` con valores exactos: su scenario es estadístico y se marca
  como tal (rango 0 a 99 y dos lecturas distintas).
- No pruebes `roars_near` ni `last_roar` (F6-06).
- No toques `crates/*/src/`; el banco se basa en `examples/` y añade sus propios
  archivos bajo `examples/terrains/`, `examples/objects/` y `examples/maps/`.

### Archivos previstos
- `examples/terrains/sand.te` — nuevo — segundo terreno con `resource_max` y `regen` propios, char `s`.
- `examples/objects/crystal.ob` — nuevo — segundo objeto con `resource_max` propio, char `*`.
- `examples/maps/mixed.map` — nuevo — franjas de hierba y arena con alguna roca y algún cristal.
- `examples/creatures/scout.ins` — nuevo — visión amplia; dice lo que percibe de sí, del mundo y de alrededor.
- `examples/creatures/myope.ins` — nuevo — visión mínima; dice lo que ve fuera de su radio.
- `examples/creatures/companion.ins` — nuevo — aliado quieto, para contar y buscar aliados.
- `examples/creatures/stranger.ins` — nuevo — criatura de otra facción, para contar y buscar enemigos.
- `crates/instinct-cli/tests/cli_bank/perceptions.rs` — nuevo — scenarios observables desde la terminal.
- `crates/instinct-cli/tests/cli_bank/mod.rs` — modificar — declarar el módulo nuevo.
- `crates/instinct-core/tests/bank/perceptions.rs` — nuevo — valores exactos de percepciones, `see` y `name`.
- `crates/instinct-core/tests/bank/mod.rs` — modificar — declarar el módulo nuevo.
- `examples/README.md` — modificar — escenarios S25 a S33.

## Criterios de aceptación

- [ ] **AC1** Un explorador dice su posición, su edad, su salud, su especie, su facción, el terreno bajo él y su reserva, y los valores coinciden con su cabecera y su colocación. Verif: `cargo test -p instinct-core --test bank perceptions::self_perceptions_match_placement`.
- [ ] **AC2** El explorador dice el tamaño del mapa, el tick actual y los totales de criaturas, aliados y enemigos vivos sin contarse a sí mismo. Verif: `cargo test -p instinct-core --test bank perceptions::world_perceptions_count_others`.
- [ ] **AC3** Los conteos de cercanía (aliados, enemigos y objetos dentro del radio) excluyen la casilla propia y coinciden con lo visible en el mapa. Verif: `cargo test -p instinct-core --test bank perceptions::near_counts_exclude_own_cell`.
- [ ] **AC4** Sin objetivo visible, la distancia al más cercano vale -1 y sus direcciones 0; con objetivo visible, la dirección cae en el rango -1 a 1 y acerca a él. Verif: `cargo test -p instinct-core --test bank perceptions::nearest_directions_and_distance`.
- [ ] **AC5** Con dos objetivos a la misma distancia, el más cercano elegido es el mismo en dos turnos seguidos mientras nada cambie. Verif: `cargo test -p instinct-core --test bank perceptions::nearest_is_stable_on_ties`.
- [ ] **AC6** `see` devuelve las cinco constantes: GROUND en hierba, OBJECT en roca y en el borde fuera del mapa, ALLY en la propia casilla, ENEMY en la contraria y NONE más allá del radio de visión. Verif: `cargo test -p instinct-core --test bank perceptions::see_returns_the_five_constants`.
- [ ] **AC7** `name` devuelve la especie con criatura, el nombre del objeto, el del terreno y el texto vacío fuera del radio o del mapa. Verif: `cargo test -p instinct-core --test bank perceptions::name_returns_species_object_terrain_or_empty`.
- [ ] **AC8** `random` da un entero entre 0 y 99 y dos lecturas seguidas son distintas. Verif: `cargo test -p instinct-core --test bank perceptions::random_is_in_range_and_advances`.
- [ ] **AC9** Puertas: `cargo test --workspace` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all --check` exit 0 · `git status` sin cambios tras el commit. Verif: los cuatro comandos.

## Contexto

**Ya existe** (F6-01 y F6-02): la raíz `examples/` con hierba y roca, los objetivos de
prueba y la convención de scenario. **Contratos que debes respetar**: los de
`PHASE.md`, incluido el catálogo de chars del banco (Sand con `s`, Crystal con
`*`, sin colisiones). **Ojo**: las percepciones se rellenan al empezar el turno,
así que una criatura que se mueve percibe su nueva posición en el turno siguiente;
`see` y `name` solo responden dentro del radio de visión de la criatura.

### Leer primero
- `crates/instinct-core/src/perceptions/builtins/core.rs` — percepciones propias y de mundo: qué devuelve cada una.
- `crates/instinct-core/src/perceptions/builtins/near.rs` — conteos y más cercanos de cercanía.
- `crates/instinct-core/src/perceptions/builtins/social.rs` — `roars_near` y `last_roar` (los prueba F6-06; aquí solo sabemos que existen).
- `crates/instinct-core/src/functions/see.rs` — las cinco constantes y el orden de comprobación.
- `crates/instinct-core/src/functions/name.rs` — de dónde sale el nombre devuelto.
- `crates/instinct-core/src/resources/terrain.rs` — claves válidas de un terreno (Sand con `resource_max` y `regen`).
- `crates/instinct-core/src/resources/object.rs` — claves válidas de un objeto (Crystal solo con `resource_max`).
- `docs/ambiguedades.md` — A14, A15 y A16: rugido propio excluido, empates y casilla propia.

## Especificación

La técnica común del issue: cada criatura del banco tiene un programa que dice
por el registro el valor que percibe, de modo que el resultado esperado sea una
línea de registro concreta. Donde el valor exacto dependa de la semilla
(`random`), el scenario es estadístico y el README lo marca.

| # | Regla (enunciado) | Escenario | Resultado esperado |
|---|-------------------|-----------|--------------------|
| S25 | 2.6 percepciones sobre sí misma | Explorador en Arena | dice salud, edad, visión, especie, facción, x, y, terreno y reserva de su casilla |
| S26 | 2.6 percepciones del mundo | Explorador con compañeros | dice ancho, alto, tick y totales de criaturas, aliados y enemigos |
| S27 | 2.6 cercanía | Explorador con aliados, enemigos, roca y cristal alrededor | dice los conteos dentro de su radio, sin contarse |
| S28 | 2.6 más cercano | Explorador con un objeto y un enemigo a la vista | dice la distancia y la dirección al más cercano de cada tipo |
| S29 | 2.6 ver y borde | Mióope junto a una roca y al borde | `see` da OBJECT en la roca y en el borde, GROUND en la hierba, NONE más allá de su radio |
| S30 | 2.6 ver aliados | Explorador con un compañero de su facción | `see` da ALLY en la casilla del compañero y en la propia |
| S31 | 2.6 ver enemigos | Explorador con un desconocido de otra facción | `see` da ENEMY en la casilla del desconocido |
| S32 | 2.6 `name` | Explorador en el mapa mixto | dice la especie, la roca, el cristal, la hierba, la arena y el vacío fuera del radio |
| S33 | 2.6 `random` | Cualquier criatura que lo lea dos veces | entero entre 0 y 99, con dos lecturas distintas (estadístico) |

El mapa mixto se construye con franjas de hierba y arena para que `name` y
`terrain_here` distingan dos terrenos sin salirse de la base: todo terreno se pisa.

### Pasos sugeridos
1. Crea Sand y Crystal y comprueba que cargan sin diagnósticos junto a la hierba y la roca.
2. Escribe el explorador y comprueba a mano una línea de registro por grupo de percepciones.
3. Escribe primero las pruebas del núcleo con valores exactos y luego las de terminal con la línea de registro.
4. Marca el scenario de `random` como estadístico en el README.
5. Rellena el README con la salida real.

## Tests

Nivel: integración. En `bank/perceptions.rs` del núcleo, los valores exactos de
cada percepción y de `see`/`name` contra el mundo construido desde `examples/`, y el
rango de `random`. En `cli_bank/perceptions.rs`, las líneas de registro que la
criatura emite con esos valores, más la rejilla final. No se mockea nada: mundo y
simulación reales, binario real de la CLI. Semilla fija salvo el scenario
estadístico.

## Verificación

- `cargo test -p instinct-core --test bank perceptions` → valores de percepciones, `see` y `name` correctos.
- `cargo test -p instinct-cli --test cli_bank perceptions` → líneas de registro esperadas.
- `cargo run -p instinct-cli -- run --root examples --map Mixed --seed 1 --ticks 2 --place Scout@4,3 --place Companion@5,3 --place Stranger@6,3` → exit 0 con las líneas de registro documentadas.
- `cargo test --workspace` → sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- **Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin cambios en `crates/*/src/` + README sin resultados inventados.

## Si te bloqueas

Si una percepción no tiene forma de observarse desde la terminal, el resultado
del README se limita a lo observable y la prueba del núcleo fija el valor exacto.
Si el empate de «más cercano» no sale estable, documéntalo como desviación
posible frente a A15 y repórtalo; no cambies el recorrido. Decisiones que NO son
tuyas: cambiar percepciones o funciones, tocar `crates/*/src/`,
añadir terrenos con comportamiento.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación