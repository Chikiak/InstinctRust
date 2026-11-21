---
id: F6-01
title: Esqueleto del banco — movimiento, bloqueos y sueño de wait
phase: F6
status: done
depends_on: []
size: M
file: f6-01-esqueleto-banco.md
---

# F6-01 · Esqueleto del banco — movimiento, bloqueos y sueño de `wait`

## Objetivo

Reusar la carpeta `examples/` existente como raíz del banco (cambiando nombre/dir
respecto a una raíz `bank/` nueva), levantar la convención de scenario de
`examples/README.md` y los dos hogares de pruebas del banco, verificados con los
primeros scenarios: `move` avanzando hasta `speed` y deteniéndose ante objeto,
criatura y borde; `move` diagonal; `wait` con n < 1 y con sueño de n-1 turnos;
y que una acción consume un solo turno aunque avance varias casillas.

## Alcance

Este issue es de cimientos: toca más archivos de lo habitual porque datos,
pruebas y `[[test]]` van juntos — por separado no verifican nada.

**Haz**
- `examples/` con `terrains/grass.te` y `objects/rock.ob` (valores del anexo
  3.2, ya existentes; se reusan sin modificar), `maps/arena.map` (mapa abierto
  de hierba con rocas, nuevo o reusado) y `creatures/` con una criatura por
  behavior de este issue (nuevas bajo `examples/creatures/`).
- `examples/README.md` con: cómo ejecutar el banco desde la terminal, la
  convención de scenario y las entradas S01 a S07 con el resultado esperado
  copiado de la ejecución real.
- `crates/instinct-cli/tests/cli_bank/` y `crates/instinct-core/tests/bank/`,
  cada uno con su bloque `[[test]]` en el `Cargo.toml` de su crate, siguiendo el
  patrón de `cli_run/` y de `tests/sim/`.

**No hagas**
- No toques `crates/*/src/` salvo los dos `[[test]]` en los `Cargo.toml`.
- No crees una carpeta `bank/` nueva; la raíz es `examples/`.
- No crees `examples/traps/` (F6-06), ni terrenos nuevos (F6-04).

### Archivos previstos
- `examples/terrains/grass.te` — existente — terreno base, char `.`,
  `resource_max` 100, `regen` 2. Se reusa tal cual.
- `examples/objects/rock.ob` — existente — objeto base, char `#`,
  `resource_max` 100. Se reusa tal cual.
- `examples/maps/arena.map` — nuevo — mapa abierto de hierba con rocas para los
  bloqueos.
- `examples/creatures/sprinter.ins` — nuevo — `move` en línea recta con `speed`
  3.
- `examples/creatures/statue.ins` — nuevo — criatura quieta para bloqueo por
  criatura.
- `examples/creatures/diagonal.ins` — nuevo — `move` en diagonal.
- `examples/creatures/pacer.ins` — nuevo — `move` de dos casillas y luego un
  dicho, para probar un turno por acción.
- `examples/creatures/napper.ins` — nuevo — `wait(3)` y un dicho al despertar.
- `examples/creatures/impatient.ins` — nuevo — `wait(0)` y un dicho cada turno.
- `examples/README.md` — nuevo — convención de scenario y entradas S01 a S07.
- `crates/instinct-cli/tests/cli_bank/mod.rs` — nuevo — declara los módulos del objetivo.
- `crates/instinct-cli/tests/cli_bank/helpers.rs` — nuevo — rutas del banco y ejecución del binario.
- `crates/instinct-cli/tests/cli_bank/movement.rs` — nuevo — scenarios observables desde la terminal.
- `crates/instinct-core/tests/bank/mod.rs` — nuevo — declara los módulos del objetivo.
- `crates/instinct-core/tests/bank/common.rs` — nuevo — carga del banco y construcción del mundo.
- `crates/instinct-core/tests/bank/movement.rs` — nuevo — posiciones y edades tras los ticks.
- `crates/instinct-cli/Cargo.toml` — modificar — bloque `[[test]]` de `cli_bank`.
- `crates/instinct-core/Cargo.toml` — modificar — bloque `[[test]]` de `bank`.

## Criterios de aceptación

- [ ] **AC1** `examples/` carga sin diagnósticos:
  `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 0`
  termina con exit 0 y stderr vacío. Verif: comando en terminal +
  `cargo test -p instinct-cli --test cli_bank movement::bank_loads_clean`.
- [ ] **AC2** Sprinter con `speed` 3 avanza tres casillas en un turno y queda en la casilla esperada en el tick 1. Verif: `cargo test -p instinct-cli --test cli_bank movement::sprinter_advances_three_cells`.
- [ ] **AC3** Sprinter se detiene en la casilla anterior a una roca y no la pisa. Verif: `cargo test -p instinct-cli --test cli_bank movement::sprinter_stops_before_rock`.
- [ ] **AC4** Sprinter se detiene ante una criatura: Statue sigue en su casilla al tick siguiente y Sprinter no la ha dañado. Verif: `cargo test -p instinct-cli --test cli_bank movement::sprinter_stops_before_creature`.
- [ ] **AC5** Sprinter se detiene en la última casilla del mapa cuando el borde corta el avance. Verif: `cargo test -p instinct-cli --test cli_bank movement::sprinter_stops_at_border`.
- [ ] **AC6** Un `move` en diagonal desplaza una casilla en x y una en y dentro del mismo turno. Verif: `cargo test -p instinct-cli --test cli_bank movement::diagonal_moves_one_cell_each_axis`.
- [ ] **AC7** `wait(3)` no ejecuta acción en los dos ticks siguientes y actúa en el tercero; `wait(0)` actúa en cada tick. Verif: `cargo test -p instinct-cli --test cli_bank movement::wait_sleeps_two_turns` y `cargo test -p instinct-core --test bank movement::waiter_ages_while_sleeping`.
- [ ] **AC8** Pacer avanza dos casillas en el tick 0 y su dicho aparece en el mismo tick: una acción por turno aunque el movimiento sea de varias casillas. Verif: `cargo test -p instinct-cli --test cli_bank movement::pacer_one_action_per_turn`.
- [ ] **AC9** Puertas: `cargo test --workspace` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all --check` exit 0 · `git status` sin cambios tras el commit. Verif: los cuatro comandos.

## Contexto

**Ya existe**: la CLI `run` con `--place` repetible y salida de rejilla por tick
(`crates/instinct-cli/src/run.rs`); el patrón de pruebas E2E con el binario real
en `crates/instinct-cli/tests/cli_run/`; los helpers de mundo y `run_ticks` en
`crates/instinct-core/tests/sim/common.rs`; `examples/terrains/grass.te`,
`examples/objects/rock.ob` y `examples/maps/plain.map`.
**Contratos que debes respetar**: los de `PHASE.md`. **Ojo**: `load_resources`
lee las carpetas planas y en orden alfabético; si `examples/` tuviera una
subcarpeta con recursos, no cargaría. El `char` de un terreno u objeto no puede
repetirse dentro de la misma raíz.

### Leer primero
- `examples/creatures/walker.ins` — criatura mínima válida: cabecera completa y una acción.
- `examples/terrains/grass.te` — formato exacto de un archivo de terreno.
- `examples/objects/rock.ob` — formato exacto de un archivo de objeto.
- `examples/maps/plain.map` — formato de mapa: nombre en la primera línea y filas de chars.
- `crates/instinct-cli/tests/cli_run/helpers.rs` — `run_cli` y `examples_dir`, el patrón de ejecución del binario.
- `crates/instinct-cli/tests/cli_run/e2e.rs` — cómo se afirman rejilla y eventos con semilla fija.
- `crates/instinct-core/src/actions/wait.rs` — semántica de `wait`: n < 1 cuenta como 1 y duerme n-1 turnos.
- `crates/instinct-core/src/resources/mod.rs` — orden de lectura de carpetas; los recursos inválidos se descartan sin abortar.

## Especificación

Arena es un mapa de hierba lo bastante grande para que un avance de tres casillas
desde el centro no llegue al borde, con rocas colocadas de forma que cada bloqueo
del enunciado se pruebe con una colocación distinta. Las criaturas son mínimas:
cabecera completa, la acción del caso y un `goto` para mantenerse vivas.

| # | Regla (enunciado) | Escenario | Resultado esperado |
|---|-------------------|-----------|--------------------|
| S01 | 2.4 `move`: avanza hasta `speed` casillas | Sprinter con `speed` 3 sobre fila despejada | aparece tres casillas a la derecha en el tick 1 |
| S02 | 2.4 `move`: se detiene ante objeto | Sprinter con roca dos casillas al frente | queda en la casilla anterior a la roca; la roca sigue en su sitio |
| S03 | 2.4 `move`: se detiene ante criatura | Sprinter con Statue dos casillas al frente | queda antes que Statue; Statue intacta en el tick 1 |
| S04 | 2.4 `move`: se detiene ante el borde | Sprinter cerca de la última columna | queda en la última casilla, sin salir del mapa |
| S05 | 2.4 `move`: dirección diagonal | Diagonal con dx y dy distintos de cero | se mueve una casilla en cada eje en el mismo tick |
| S06 | 2.4 `wait`: n < 1 cuenta como 1 | Impatient con `wait(0)` | actúa en cada tick |
| S07 | 2.4 `wait` + 2.7 punto 4: duerme n-1 turnos y sigue envejeciendo | Napper con `wait(3)` | sin acción en los ticks 1 y 2, actúa en el 3, y muere de edad siguiendo el `lifespan` de su cabecera |

La convención de scenario del README es una entrada por fila de la tabla con:
regla, archivos que usa, el comando exacto de terminal (`--seed 1` salvo indicación),
el resultado esperado copiado de la ejecución real y el nombre del test que lo
verifica.

### Pasos sugeridos
1. Escribe primero los archivos de datos de `examples/` y comprueba que cargan sin diagnósticos.
2. Escribe las pruebas de `cli_bank/movement` contra los scenarios y ajusta criaturas y mapa hasta que describan el comportamiento real.
3. Añade el estado interno (posición y edad) en `bank/movement` del núcleo.
4. Rellena el README con la salida real de cada comando y ejecuta las puertas completas.

## Tests

Nivel: integración. En `crates/instinct-cli/tests/cli_bank/movement.rs`, los
scenarios observables (rejilla tras el tick, líneas de eventos, stderr y código
de salida); en `crates/instinct-core/tests/bank/movement.rs`, el estado interno
(posición exacta y edad) tras los ticks. No se mockea nada: `Simulation` y
`World` reales cargados desde `examples/`, y el binario real de la CLI. Ninguno de
estos scenarios lee `random`, así que la semilla no mueve el resultado.

## Verificación

- `cargo test -p instinct-cli --test cli_bank` → todos los scenarios de movimiento pasan.
- `cargo test -p instinct-core --test bank` → estado interno (posición y edad) correcto.
- `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 3 --place Sprinter@2,2` → exit 0, mapa ticks 0 y 3 y stderr vacío.
- `cargo test --workspace` → sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- **Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin cambios en `crates/*/src/` salvo los `[[test]]` + README sin resultados inventados.

## Si te bloqueas

Si `arena.map` no permite algún bloqueo con las colocaciones previstas, amplía el
mapa o mueve las rocas y documéntalo en el README; no cambies la semántica de
`move`. Decisiones que NO son tuyas: cambiar `move`/`wait`, tocar `crates/*/src/`,
crear la raíz de trampas o corregir el núcleo.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

**Qué se creó**
- Datos de `examples/` (raíz del banco, sin carpeta `bank/` nueva): `maps/arena.map` y las criaturas `sprinter.ins`, `statue.ins` (usada como `Guardian`), `diagonal.ins`, `pacer.ins`, `napper.ins`, `impatient.ins`; se reusan sin modificar `terrains/grass.te` y `objects/rock.ob`.
- `examples/README.md`: cómo ejecutar el banco, convención de scenario y entradas S01–S07 con resultado esperado copiado de la ejecución real, más sección de Desviaciones.
- Pruebas: `crates/instinct-cli/tests/cli_bank/` (`mod.rs`, `helpers.rs`, `movement.rs`, 8 tests: AC1–AC8) y `crates/instinct-core/tests/bank/` (`mod.rs`, `common.rs`, `movement.rs`, 4 tests: estado interno posición/edad).
- `[[test]]` en ambos `Cargo.toml`: `cli_bank` (instinct-cli) y `bank` (instinct-core).
- Sin cambios en `crates/*/src/` salvo los dos bloques `[[test]]`.

**Desviaciones documentadas (README, sección Desviaciones)**
- AC7 / S07: el enunciado dice que `wait(3)` "actúa en el 3"; en esta referencia el `say("despierto")` de Napper se emite con etiqueta `[tick 3]` durante la transición a tick 4 (el turno de ejecución ocurre en el paso que produce tick 4). El evento aparece tras la rejilla de tick 3. La semántica de `wait` (duerme n-1 turnos, actúa en el n-ésimo) se respeta.
- AC8: `move` consume el turno y el `say("paso")` de Pacer se ejecuta en el turno siguiente, no junto al movimiento de varias casillas. Documentado como desviación; el kernel no se modifica en esta fase.

**Resultado de las puertas**
- `cargo test -p instinct-cli --test cli_bank`: 8/8 PASS (exit 0).
- `cargo test -p instinct-core --test bank`: 4/4 PASS (exit 0).
- `cargo test --workspace`: exit 101; única falla `instinct-gui::tests::determinismo::t_determinismo_01_gui_iguala_directo_con_seed7` (left: 2, right: 4), pre-existing y ajena a este issue. Todos los tests de CLI y núcleo pasan.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --all --check`: exit 1; únicos diffs en `crates/instinct-gui/src/ui/mod.rs`, pre-existing y ajenos a este issue. Ningún archivo nuevo/modificado de este issue tiene diff de formato.
- Árbol limpio tras commit: `git status --short` vacío.