---
id: F6-02
title: Golpes, consumo y reservas
phase: F6
status: done
depends_on: [F6-01]
size: M
file: f6-02-golpes-consumo-reservas.md
---

# F6-02 · Golpes, consumo y reservas

## Objetivo

Scenarios de `attack` y `consume` contra cada tipo de objetivo del enunciado
(criatura, objeto, terreno, la propia casilla con `(0, 0)` y fuera del mapa) y de
las leyes de fin de tick que los acompañan: la criatura golpeada hasta vida 0
muere en el acto y deja solo terreno, el terreno regenera sin pasarse de
`resource_max`, el objeto agotado desaparece al final del tick, y toda criatura
pierde un punto de vida y suma uno de edad.

## Alcance

**Haz**
- Criaturas del banco para este issue: quien golpea a una criatura, quien golpea
  sin matarla, quien golpea y consume rocas, quien consume la hierba bajo sí
  misma, y las víctimas quietas (frágil y resistente).
- Scenarios en `examples/README.md` con el comando exacto y el resultado observado,
  más las aserciones de estado interno (reservas, vidas, edades) que la terminal
  no muestra.
- Pruebas en `crates/instinct-cli/tests/cli_bank/combat.rs` (eventos, rejilla,
  stderr, exit) y en `crates/instinct-core/tests/bank/combat.rs` (reservas de
  terreno y objeto, vidas, edades y casillas liberadas).

**No hagas**
- No pruebes `reproduce` ni el ciclo de turnos (F6-03) ni `see`/`name` (F6-04).
- No crees los terrenos Sand ni Crystal: este issue solo usa hierba y roca.
- No toques `crates/*/src/`; el banco se basa en `examples/` y su `arena.map`.

### Archivos previstos
- `examples/creatures/brawler.ins` — nuevo — golpea a la criatura de la casilla contigua con daño suficiente para matarla.
- `examples/creatures/bruiser.ins` — nuevo — golpea con daño insuficiente para matar.
- `examples/creatures/miner.ins` — nuevo — golpea la roca contigua.
- `examples/creatures/grazer.ins` — nuevo — consume el terreno de su propia casilla.
- `examples/creatures/forager.ins` — nuevo — consume la roca contigua curándose.
- `examples/creatures/victim.ins` — nuevo — criatura quieta y frágil, para morir de un golpe.
- `examples/creatures/tough.ins` — nuevo — criatura quieta y resistente, para sobrevivir al golpe.
- `crates/instinct-cli/tests/cli_bank/combat.rs` — nuevo — scenarios observables desde la terminal.
- `crates/instinct-cli/tests/cli_bank/mod.rs` — modificar — declarar el módulo nuevo.
- `crates/instinct-core/tests/bank/combat.rs` — nuevo — estado interno tras los ticks.
- `crates/instinct-core/tests/bank/mod.rs` — modificar — declarar el módulo nuevo.
- `examples/README.md` — modificar — escenarios S08 a S16.

## Criterios de aceptación

- [ ] **AC1** `attack` con daño 10 sobre una víctima de vida 5 la mata en el mismo tick y la casilla queda solo con terreno en el tick siguiente. Verif: `cargo test -p instinct-cli --test cli_bank combat::attack_kills_and_frees_cell`.
- [ ] **AC2** `attack` con daño 5 sobre una víctima de vida 20 la deja en 15 y viva. Verif: `cargo test -p instinct-core --test bank combat::attack_wounds_without_killing`.
- [ ] **AC3** `attack` a una roca le quita el daño a su reserva (100 menos el daño) y el evento de daño nombra la casilla golpeada. Verif: `cargo test -p instinct-core --test bank combat::attack_reduces_object_reserve` y `cargo test -p instinct-cli --test cli_bank combat::attack_object_emits_damaged`.
- [ ] **AC4** `attack` a una casilla de hierba le quita reserva al terreno y la casilla sigue siendo hierba en la rejilla. Verif: `cargo test -p instinct-core --test bank combat::attack_reduces_terrain_reserve`.
- [ ] **AC5** Golpear la propia casilla con `(0, 0)` daña el terreno bajo la criatura, no a la criatura, y esta sigue en su casilla. Verif: `cargo test -p instinct-core --test bank combat::attack_own_cell_hits_terrain`.
- [ ] **AC6** `attack` y `consume` con destino fuera del mapa no hacen nada: sin evento de daño y con reservas intactas. Verif: `cargo test -p instinct-cli --test cli_bank combat::attack_out_of_map_is_silent` y `cargo test -p instinct-core --test bank combat::consume_out_of_map_changes_nothing`.
- [ ] **AC7** `consume` cura al atacante sin tope: su vida queda por encima del valor de su cabecera. Verif: `cargo test -p instinct-core --test bank combat::consume_heals_above_header_health`.
- [ ] **AC8** Fin de tick: el terreno recupera su `regen` sin pasar de `resource_max`; el objeto con reserva 0 o menos desaparece y su casilla queda libre; cada criatura pierde 1 de vida y suma 1 de edad. Verif: `cargo test -p instinct-core --test bank combat::end_of_tick_laws`.
- [ ] **AC9** Puertas: `cargo test --workspace` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all --check` exit 0 · `git status` sin cambios tras el commit. Verif: los cuatro comandos.

## Contexto

**Ya existe** (F6-01): la raíz `examples/` con hierba y roca y el mapa Arena, los
objetivos de prueba `cli_bank` y `bank`, y la convención de scenario del README.
**Contratos que debes respetar**: los de `PHASE.md`. **Ojo**: las reservas de
terreno y objeto son estado interno; desde la terminal solo se ve el daño como
evento y la desaparición del objeto en la rejilla, así que el README separa
«resultado observable» de «estado interno que fija la prueba del núcleo». El
daño a una casilla vacía es siempre al terreno de esa casilla.

### Leer primero
- `crates/instinct-core/src/actions/combat.rs` — `strike_cell`: objetivo fuera del mapa, `(0, 0)` sobre el terreno propio y daño a criatura, objeto y terreno.
- `crates/instinct-core/src/actions/consume.rs` — cómo se suma la curación al atacante.
- `crates/instinct-core/src/world/laws.rs` — leyes de fin de tick: vida, edad, regeneración con tope y objetos agotados.
- `crates/instinct-core/src/event/display.rs` — texto exacto de los eventos `Damaged` y `Died`.
- `crates/instinct-core/tests/sim/common.rs` — construcción de mundo y avance de ticks para las pruebas de estado interno.
- `crates/instinct-cli/tests/cli_run/e2e.rs` — patrón de aserciones sobre rejilla y eventos con el binario real.

## Especificación

Cada escenario coloca una o dos criaturas sobre Arena, corre pocos ticks con
`--seed 1` y comprueba un resultado. Las criaturas son mínimas: cabecera
completa, la acción del caso y un `goto` para mantenerse vivas.

| # | Regla (enunciado) | Escenario | Resultado esperado |
|---|-------------------|-----------|--------------------|
| S08 | 2.4 `attack` a criatura + 3.4 | Brawler contra víctima frágil contigua | víctima muere en el acto; su casilla queda solo con terreno |
| S09 | 2.4 `attack` a criatura | Bruiser contra víctima resistente | la víctima sobrevive con la vida reducida en el daño |
| S10 | 2.4 `attack` a objeto + 3.1 | Miner contra la roca contigua | la reserva de la roca baja el daño; evento de daño con la casilla |
| S11 | 2.4 `attack` a terreno | Miner contra casilla de hierba | la reserva del terreno baja; la casilla sigue siendo hierba |
| S12 | 2.4 `attack` con `(0, 0)` | Grazer sobre su propia casilla | el terreno bajo la criatura pierde reserva; la criatura no se daña |
| S13 | 3.4 fuera del mapa | Brawler junto al borde golpeando hacia fuera | nada cambia: sin daño, sin muerte, turno consumido |
| S14 | 2.4 `consume` cura sin tope | Forager contra la roca contigua | reserva de la roca reducida y vida del atacante por encima de la de su cabecera |
| S15 | 3.4 fin de tick, terreno | consumo repetido de la misma hierba | la reserva se regenera hasta el máximo y no pasa de él |
| S16 | 3.4 fin de tick, objeto | roca golpeada hasta agotarse | la roca desaparece al final del tick y la casilla queda libre |
| S17 | 3.4 fin de tick, criatura | cualquier criatura tranquila | pierde 1 de vida y suma 1 de edad por tick |

### Pasos sugeridos
1. Escribe las criaturas y comprueba a mano un golpe de cada tipo con el comando del escenario.
2. Escribe las pruebas de `cli_bank/combat` sobre eventos y rejilla.
3. Escribe las pruebas de `bank/combat` sobre reservas, vidas y edades.
4. Rellena el README con la salida real y cita los tests.

## Tests

Nivel: integración. En `cli_bank/combat.rs`, comportamientos observables: la
víctima desaparece y su casilla queda con terreno, el evento de daño nombra la
casilla correcta, el golpe fuera del mapa no produce ningún evento. En
`bank/combat.rs`, estado interno: reservas de terreno y objeto antes y después,
vida y edad de las criaturas, casilla liberada al desaparecer el objeto. No se
mockea nada: `Simulation` y `World` reales cargados desde `examples/`, y el binario
real de la CLI. Ningún scenario lee `random`.

## Verificación

- `cargo test -p instinct-cli --test cli_bank combat` → escenarios de combate observables pasan.
- `cargo test -p instinct-core --test bank combat` → reservas, vidas y edades correctas.
- `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 2 --place Brawler@2,2 --place Victim@3,2` → exit 0, víctima muerta en el tick 0 y casilla libre en el tick 1.
- `cargo test --workspace` → sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- **Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin cambios en `crates/*/src/` + README sin resultados inventados.

## Si te bloqueas

Si algún resultado del enunciado no se cumple en la referencia, no lo parches:
documéntalo en Desviaciones del README, repórtalo en las notas y déjalo para
F7. Decisiones que NO son tuyas: cambiar `attack`/`consume`, tocar `crates/*/src/`,
añadir movimientos o percepciones.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación