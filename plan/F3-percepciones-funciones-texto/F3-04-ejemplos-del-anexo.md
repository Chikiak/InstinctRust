---
id: F3-04
title: Los cuatro ejemplos del anexo
phase: F3
status: todo
depends_on: [F3-03]
size: M
---

# F3-04 · Los cuatro ejemplos del anexo

## Objetivo

Cerrar el entregable observable de la fase: los cuatro ejemplos del anexo (`hobbit.ins`, `ent.ins`,
`dwarf.ins`, `uruk.ins`) en `examples/creatures/`, compilando sin diagnostics y mostrando en la
terminal los comportamientos que el anexo describe, con dos corridas con la misma semilla dando
exactamente la misma salida.

## Alcance

**Haz**
- Crear los cuatro archivos `.ins` en `examples/creatures/` con el contenido del anexo
      (`docs/enunciado.md`, sección "Anexo"), comentarios incluidos y sin cambios de semántica.
- Pruebas automáticas por criatura: compilación sin diagnostics y un escenario de simulación con
      semilla fija que afirme el comportamiento descrito en el anexo.
- Prueba E2E de terminal con las cuatro especies colocadas en el mismo mapa: exit 0, stderr vacío y
      salida idéntica en dos ejecuciones con la misma semilla.
- Ningún escenario emite evento `RuntimeError`.
- Suite completa, clippy y fmt en verde.

**No hagas**
- No cambies ninguna criatura de `examples/creatures/` anterior a esta fase.
- No extiendas el lenguaje ni añadas acciones, funciones o percepciones nuevas.
- No ajustes el comportamiento del mundo para que un ejemplo "encaje": si un ejemplo no compila o
      no se comporta como el anexo, es un fallo que reportar, no un cambio de semántica.
- No escribas mapas nuevos: `examples/maps/plain.map` y `examples/objects/rock.ob` bastan para los
      cuatro.

### Archivos previstos
- `examples/creatures/hobbit.ins` — nuevo — el Hobbit del anexo.
- `examples/creatures/ent.ins` — nuevo — el Ent del anexo.
- `examples/creatures/dwarf.ins` — nuevo — el Enano del anexo.
- `examples/creatures/uruk.ins` — nuevo — el Uruk de la sección 2.1.
- `crates/instinct-core/tests/annex_examples_tests.rs` — nuevo — compilación y escenarios por
      criatura.
- `crates/instinct-cli/tests/cli_annex.rs` — nuevo — E2E de terminal con las cuatro especies y
      determinismo.

## Criterios de aceptación

- [ ] **AC1** Los cuatro archivos compilan sin diagnostics con el catálogo real. Verif: `test_annex_creatures_compile` | `cargo test -p instinct-core --test annex_examples_tests`.
- [ ] **AC2** El Hobbit, con un enemigo a la vista, dice la frase de huida del anexo y se aleja de él (la distancia al enemigo crece entre ticks). Verif: `test_hobbit_flees_and_speaks` | `cargo test -p instinct-core --test annex_examples_tests`.
- [ ] **AC3** El Ent come de su terreno (evento `Damaged` de 3 en su propia casilla) y no abandona su casilla mientras nada lo aparte. Verif: `test_ent_feeds_and_stays` | `cargo test -p instinct-core --test annex_examples_tests`.
- [ ] **AC4** El Enano junto a una roca la mina (evento `Damaged` de 20 en la casilla de la roca) y la casilla queda libre cuando la reserva del objeto se agota. Verif: `test_dwarf_mines_adjacent_rock` | `cargo test -p instinct-core --test annex_examples_tests`.
- [ ] **AC5** El Uruk deambula con `random`, consume al enemigo adyacente cuando está a distancia 1 y se reproduce cuando se dan sus condiciones, sin ningún `RuntimeError` en 30 ticks. Verif: `test_uruk_hunts_and_breeds` | `cargo test -p instinct-core --test annex_examples_tests`.
- [ ] **AC6** El E2E de terminal con las cuatro especies colocadas sale con código 0 y stderr vacío. Verif: `test_cli_runs_all_annex_examples` | `cargo test -p instinct-cli --test cli_annex`.
- [ ] **AC7** Dos ejecuciones del mismo escenario con la misma semilla producen una salida idéntica. Verif: `test_cli_annex_is_deterministic` | `cargo test -p instinct-cli --test cli_annex`.
- [ ] **AC8** Los comandos `check` de la CLI aceptan los cuatro archivos. Verif: `cargo run -p instinct-cli -- check examples/creatures/hobbit.ins` (y `ent`, `dwarf`, `uruk`) → imprime `OK` y exit 0.
- [ ] **AC9** Suite completa sin regresiones y puertas de calidad en verde. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: la fase F3 completa (31 percepciones, `see`, `name`, cinco constantes); el anexo con
el contenido literal de las cuatro criaturas en `docs/enunciado.md`; `examples/` con `plain.map`
(7 filas x 20 columnas), `grass.te` y `rock.ob`; el ayudante `examples_dir()` y el patrón de
escenarios E2E en `crates/instinct-cli/tests/cli_run.rs`; `cargo run -p instinct-cli -- run` con
`--place Especie@x,y`, `--seed`, `--ticks` y `--every`.

**Contratos que debes respetar**: los de `PHASE.md#contratos-compartidos`, en particular el
determinismo con semilla y el hecho de que el Ent y el Enano dependen de percepciones y funciones
cerradas en F3-01 a F3-03.

**Ojo**: las pruebas leen los `.ins` de `examples/creatures/` con una ruta relativa al directorio
del crate (`CARGO_MANIFEST_DIR`), como hace `cli_run.rs` con `examples_dir()`. El comportamiento
afirmado debe ser el que describe el anexo, no el que salga: si el anexo dice que el Hobbit huye,
la prueba afirma que huye. La reserva del terreno es de catálogo, así que varios Ents comiendo del
mismo terreno comparten reserva: coloca las criaturas de cada escenario de forma que el
comportamiento esperado no dependa de esa compartición salvo que la prueba lo contemple.

### Leer primero
- `docs/enunciado.md` — la sección "Anexo", con el código literal de las cuatro criaturas y las
      dos notas finales sobre el Ent y el Hobbit.
- `crates/instinct-cli/tests/cli_run.rs` — `examples_dir()`, `instinct_cli_binary()`, `run_cli` y
      `rows_after_tick`: los ayudantes que reutiliza el E2E.
- `crates/instinct-core/tests/sim_tests.rs` — cómo se construye un mundo, se colocan criaturas y se
      afirman eventos tick a tick.
- `examples/creatures/` — las criaturas existentes, para mantener el estilo de cabecera y
      comentarios.

## Especificación

Contenido de los cuatro archivos: el literal del anexo, con su cabecera de cinco claves, sus
etiquetas y sus comentarios. Los comportamientos que las pruebas deben afirmar, según el anexo:

| Criatura | Comportamiento descrito | Afirmación en la prueba |
|---|---|---|
| Hobbit | pacífico, come en cuanto puede, huye de todo, busca compañía y llena la Comarca de hijos | dice la frase de huida y se aleja del enemigo; come el terreno cuando puede; se reproduce cuando ve suelo libre |
| Ent | no se mueve de su bosque, se alimenta por las raíces, avisa antes de pelear y vuelve a su sitio | consume 3 de su terreno y no cambia de casilla; ruge cuando hay enemigos cerca y no ha rugido nadie |
| Dwarf | camina sin rumbo, mina toda roca que encuentra, pide auxilio acorralado y acude al grito de los suyos | mina la roca adyacente por 20; ruge acorralado; responde al rugido de otro acercándose |
| Uruk | deambula, muerde al enemigo cercano, cría cuando puede y celebra | consume al enemigo a distancia 1; se reproduce con las condiciones del anexo |

Escenario E2E: `run --root examples --map "Plain with rocks"` con `--seed` fijo, `--ticks` 30 y las
cuatro especies colocadas en casillas distintas y libres; dos ejecuciones con la misma semilla
producen la misma salida en stdout.

### Pasos sugeridos (opcional)
1. Crea los cuatro archivos y comprueba con `check` que compilan.
2. Escribe las pruebas de compilación y los escenarios por criatura, y ajústalas solo si el
   resultado real contradice al anexo (reportándolo entonces).
3. Cierra con el E2E de terminal y el determinismo.

## Tests

Nivel: integración. Archivo nuevo `crates/instinct-core/tests/annex_examples_tests.rs` con mundo
real, catálogo real y semilla fija; un escenario por criatura afirmando eventos (`Said`,
`Damaged`, `Died`, `Roared`) y posiciones, no salida de texto. Archivo nuevo
`crates/instinct-cli/tests/cli_annex.rs` ejecutando el binario real con los ayudantes de
`cli_run.rs`. Determinismo: semilla fija en todos los escenarios y comparación byte a byte de las
dos salidas del E2E.

## Verificación
- `cargo test -p instinct-core --test annex_examples_tests` → los cuatro escenarios pasan.
- `cargo test -p instinct-cli --test cli_annex` → el E2E y el determinismo pasan.
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 7 --ticks 30
      --place Dwarf@0,1 --place Ent@5,5 --place Hobbit@10,10 --place Uruk@15,3` → exit 0 y stderr
      vacío.
- `cargo test --workspace` → sin regresiones de F0 a F3-03.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin
stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

Si uno de los cuatro ejemplos no se comporta como describe el anexo con las percepciones, funciones
y constantes ya implementadas, para y repórtalo en "Notas de implementación" con el escenario
concreto: puede ser un fallo de F3-01 a F3-03 que hay que arreglar antes de continuar. Decisiones
que NO puedes tomar tú: cambiar el contenido de los archivos respecto al anexo, cambiar la semilla
hasta que la prueba pase, o modificar la semántica del mundo.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
<!-- Lo rellena el implementador: desviaciones, hechos descubiertos, resultado de los escenarios y
cualquier f detectado en F3-01 a F3-03. -->
