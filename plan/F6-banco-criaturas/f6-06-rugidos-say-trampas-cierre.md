---
id: F6-06
title: Rugidos y say, trampas de compilación, mapas inválidos y cierre
phase: F6
status: done
depends_on: [F6-01]
size: M
file: f6-06-rugidos-say-trampas-cierre.md
---

# F6-06 · Rugidos y `say`, trampas de compilación, mapas inválidos y cierre

## Objetivo

Scenarios de `roar` y `say`: el rugido emitido en un tick es leído por las
criaturas que lo oyen —dentro de su radio de visión— en el tick siguiente, con su
texto y su conteo, y una criatura no oye su propio rugido; `say` muestra su texto
en el registro sin consumir el turno. Cierra el banco con los recursos inválidos
que no son criaturas: un mapa con un carácter que ningún terreno ni objeto
declara y un mapa con filas de distinto largo, rechazados con mensaje y línea
mientras el resto de la carga sigue disponible. Y cierra la fase: una sección
Desviaciones en el README que recoja las diferencias reales entre la referencia y
el enunciado, y la comprobación de que cada scenario del README tiene su prueba.

## Alcance

**Haz**
- Una criatura que ruge y otra que escucha y dice lo que oyó, con visión
  suficiente para oír y una variante demasiado lejos para no oír.
- Una criatura que dice y actúa en el mismo turno, para demostrar que `say` no lo
  consume.
- Una carpeta de criaturas trampa, una por clase de error de compilación del
  enunciado 2.8, que la carga rechaza informando qué pasó y en qué línea sin
  dejar de cargar las demás.
- En `examples/traps/maps/`, un mapa con carácter desconocido y uno con una fila
  dispareja; en `examples/traps/terrains/` y `examples/traps/objects/`, un archivo con
  una clave inválida de cada tipo.
- Una sección Desviaciones en el README que recoja las diferencias reales entre
  la referencia y el enunciado encontradas en la fase.
- Comprobación de cierre: cada scenario del README cita un test que existe y
  pasa, y los criterios de salida de `PHASE.md` se cumplen.
- Pruebas en `crates/instinct-cli/tests/cli_bank/roars.rs` y `cli_bank/traps.rs`
  (líneas de registro y diagnóstico, carga tolerable) y en
  `crates/instinct-core/tests/bank/roars.rs` y `bank/traps.rs` (qué rugidos
  quedan disponibles y qué recursos están disponibles tras la carga).

**No hagas**
- No pruebes la visualización del rugido en la ventana: eso es de la GUI (F5-07);
  el banco cubre el registro y las percepciones.
- No cambies el cargador ni el constructor del mundo para alinearlos con el
  «se rechaza» del enunciado 3.3: es una desviación que se documenta y va a F7.
- No reescribas scenarios de otros issues.
- No toques `crates/*/src/`; el banco se basa en `examples/` y `examples/traps/`.

### Archivos previstos
- `examples/creatures/shouter.ins` — nuevo — ruge con un texto fijo cada turno.
- `examples/creatures/listener.ins` — nuevo — dice el último rugido oído y cuántos oyó.
- `examples/creatures/singer.ins` — nuevo — dice dos textos y se mueve en el mismo turno.
- `examples/traps/terrains/grass.te` — nuevo — terreno base de la raíz de trampas.
- `examples/traps/objects/rock.ob` — nuevo — objeto base de la raíz de trampas.
- `examples/traps/maps/arena.map` — nuevo — mapa válido para el escenario de carga tolerante.
- `examples/traps/creatures/valid.ins` — nuevo — criatura válida que debe cargar.
- `examples/traps/creatures/` — nuevo — entre diez y doce archivos `.ins` trampa, uno por clase de error.
- `examples/traps/maps/unknown_char.map` — nuevo — mapa con un carácter que ningún terreno ni objeto declara.
- `examples/traps/maps/ragged.map` — nuevo — mapa con una fila de largo distinto.
- `examples/traps/terrains/bad.te` — nuevo — terreno con una clave inválida.
- `examples/traps/objects/bad.ob` — nuevo — objeto con una clave inválida.
- `crates/instinct-cli/tests/cli_bank/roars.rs` — nuevo — scenarios observables desde la terminal.
- `crates/instinct-cli/tests/cli_bank/traps.rs` — nuevo — diagnósticos y carga tolerante desde la terminal.
- `crates/instinct-cli/tests/cli_bank/mod.rs` — modificar — declarar los módulos nuevos.
- `crates/instinct-core/tests/bank/roars.rs` — nuevo — disponibilidad y alcance de los rugidos.
- `crates/instinct-core/tests/bank/traps.rs` — nuevo — especies y recursos disponibles tras la carga.
- `crates/instinct-core/tests/bank/mod.rs` — modificar — declarar los módulos nuevos.
- `examples/README.md` — modificar — escenarios S42 a S55, sección Desviaciones y comprobación de cierre.

## Criterios de aceptación

- [ ] **AC1** Un rugido emitido en un tick aparece como evento de rugido en ese tick. Verif: `cargo test -p instinct-cli --test cli_bank roars::roar_emits_event`.
- [ ] **AC2** Al tick siguiente, la oyente dice el texto del rugido y un conteo de uno; antes de que nadie ruja, dice el texto vacío y cero. Verif: `cargo test -p instinct-cli --test cli_bank roars::listener_reads_last_roar_next_tick`.
- [ ] **AC3** Una criatura no cuenta ni lee su propio rugido. Verif: `cargo test -p instinct-core --test bank roars::own_roar_is_not_heard`.
- [ ] **AC4** Un rugido emitido fuera del radio de visión de la oyente no llega: la oyente sigue diciendo texto vacío y cero. Verif: `cargo test -p instinct-core --test bank roars::roar_outside_vision_is_not_heard`.
- [ ] **AC5** La cantante dice dos textos y se mueve dentro del mismo tick: el dicho no consume turno. Verif: `cargo test -p instinct-cli --test cli_bank roars::say_does_not_consume_turn`.
- [ ] **AC6** `check` rechaza cada trampa con exit 1 y un diagnóstico con archivo y línea en stderr. Verif: `cargo test -p instinct-cli --test cli_bank traps::check_rejects_every_trap_with_line`.
- [ ] **AC7** `check` acepta la criatura válida con `OK` y exit 0. Verif: `cargo test -p instinct-cli --test cli_bank traps::check_accepts_valid_creature`.
- [ ] **AC8** Cargar `examples/traps/` con `run` sigue funcionando: la criatura válida está disponible, se puede colocar y simular, y los diagnósticos de las trampas salen por stderr con exit 0. Verif: `cargo test -p instinct-cli --test cli_bank traps::load_traps_root_keeps_valid_creature`.
- [ ] **AC9** El mapa con carácter desconocido produce un diagnóstico que nombra el carácter y la línea de su fila, y el mapa se puede simular. Verif: `cargo test -p instinct-cli --test cli_bank traps_maps::unknown_char_reports_and_loads`.
- [ ] **AC10** El mapa con una fila dispareja produce un diagnóstico con la línea de la fila que no cuadra. Verif: `cargo test -p instinct-cli --test cli_bank traps_maps::ragged_row_reports_line`.
- [ ] **AC11** Toda desviación real entre la referencia y el enunciado encontrada en la fase está en la sección Desviaciones del README con su caso concreto. Verif: leer la sección y contrastarla con las notas de los issues F6-01 a F6-06.
- [ ] **AC12** Puertas: `cargo test --workspace` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all --check` exit 0 · `git status` sin cambios tras el commit. Verif: los cuatro comandos; y `cargo test -p instinct-cli --test cli_bank` + `cargo test -p instinct-core --test bank` → exit 0 (todos los scenarios del README).

## Contexto

**Ya existe** (F6-01 a F6-05): la raíz `examples/` y `examples/traps/`, los objetivos de
prueba y la convención de scenario. **Contratos que debes respetar**: los de
`PHASE.md`, incluida la decisión A14 de `docs/ambiguedades.md` sobre el rugido
propio. **Ojo**: los rugidos rotan al final del tick, así que un rugido solo es
legible durante el tick siguiente; la oyente necesita estar dentro del radio de
visión del emisor. El diagnóstico de carácter desconocido sale dos veces, una al
cargar el mapa y otra al construir el mundo; ambas nombran el carácter. El
enunciado 3.3 dice «se rechaza»: la referencia lo conserva, y eso es la desviación
que hay que documentar.

### Leer primero
- `crates/instinct-core/src/actions/roar.rs` — qué hace rugir y qué evento emite.
- `crates/instinct-core/src/actions/say.rs` — qué hace decir y por qué no consume turno.
- `crates/instinct-core/src/world/roars.rs` — cómo se guardan los rugidos y la rotación por tick.
- `crates/instinct-core/src/perceptions/builtins/social.rs` — `roars_near` y `last_roar`.
- `crates/instinct-core/src/compile/body.rs` — qué formas de línea son válidas y qué error produce la que no encaja.
- `crates/instinct-core/src/compile/header.rs` — las cinco claves obligatorias y los valores inválidos.
- `crates/instinct-core/src/compile/validate.rs` — etiquetas duplicadas y ausentes, asignación a percepción o constante.
- `crates/instinct-core/src/compile/action.rs` — nombres y aridades desconocidos de acciones y funciones.
- `crates/instinct-core/src/resources/map.rs` — diagnóstico de carácter desconocido y de fila dispareja.
- `crates/instinct-cli/src/check.rs` — formato y códigos de salida de `check`.

## Especificación

| # | Regla (enunciado) | Escenario | Resultado esperado |
|---|-------------------|-----------|--------------------|
| S42 | 2.4 `roar` | `shouter.ins` sobre Arena | evento de rugido con su texto en el tick 0 |
| S43 | 2.4 `roar` y 2.6 | `shouter.ins` y `listener.ins` con visión suficiente | al tick 1 la oyente dice el texto y un conteo de uno |
| S44 | A14 | `shouter.ins` sola, diciendo lo que oye | dice texto vacío y cero: no se oye a sí misma |
| S45 | 2.6 alcance | `listener.ins` con visión mínima lejos de la gritona | sigue diciendo texto vacío y cero |
| S46 | 2.4 `say` | `singer.ins` | dos dichos y un movimiento en el mismo tick |
| S47 | 2.8 línea inválida | trampa de línea que no encaja | diagnóstico con la línea de la sentencia |
| S48 | 2.8 etiquetas | trampa de etiqueta inexistente y trampa de etiqueta duplicada | un diagnóstico por etiqueta, con su línea |
| S49 | 2.8 cabecera | una trampa por cada clave ausente, incluida `start:` | diagnóstico que nombra la clave que falta |
| S50 | 2.8 expresiones y asignación | trampa de expresión mal formada, de asignación a percepción y de asignación a constante | un diagnóstico con la línea de la asignación |
| S51 | 2.8 catálogo | trampa de acción desconocida, de función desconocida y de aridad incorrecta | diagnóstico que nombra el nombre desconocido o la aridad esperada |
| S52 | 2.8 carga tolerante | `run` sobre `examples/traps/` con la criatura válida | la criatura corre, exit 0 y las trampas por stderr |
| S53 | 3.3 carácter desconocido | `run` sobre la raíz de trampas | diagnóstico con el carácter y la línea; el mapa corre y la casilla queda como primer terreno |
| S54 | 3.3 filas de distinto largo | mapa con fila dispareja | diagnóstico con la línea de la fila que no cuadra |
| S55 | 3.2 terreno y objeto | archivos con clave inválida | se descartan con su mensaje; el resto de recursos carga |

Desviaciones: sección del README con las diferencias reales frente al enunciado.
La candidata segura es la de S53. Cada desviación lleva: qué dice el enunciado,
qué hace la referencia, el archivo que lo evidencia y la ambigüedad a confirmar
con los profesores (supuesto 4 del maestro).

### Pasos sugeridos
1. Escribe la gritona, la oyente y la cantante, y comprueba a mano las líneas de registro.
2. Separa la oyente hasta que quede fuera de radio y confirma que no oye.
3. Escribe las trampas una a una y confirma con `check` el mensaje y la línea reales.
4. Escribe el escenario de carga tolerante y comprueba que el exit sigue siendo 0.
5. Recorre las notas de F6-01 a F6-05 y escribe la sección Desviaciones con lo encontrado.
6. Rellena el README con la salida real de cada comando y hace la comprobación de cierre.

## Tests

Nivel: integración. En `cli_bank/roars.rs`: las líneas de registro del rugido, del
dicho y de lo oído. En `bank/roars.rs`: qué rugidos están disponibles al empezar
cada tick y para qué criaturas. En `cli_bank/traps.rs`: `check` sobre cada archivo
trampa (exit 1, diagnóstico con línea) y sobre el válido (exit 0, `OK`); `run`
sobre la raíz de trampas (exit 0, criatura válida simulando, diagnósticos en
stderr); y la raíz válida sigue limpia. En `bank/traps.rs`: tras `load_resources`,
la especie válida está y las trampas no. No se mockea nada: cargador real del
núcleo y binario real de la CLI.

## Verificación

- `cargo test -p instinct-cli --test cli_bank roars` → líneas de registro esperadas.
- `cargo test -p instinct-core --test bank roars` → disponibilidad y alcance correctos.
- `cargo test -p instinct-cli --test cli_bank traps` → diagnósticos, exits y carga tolerante.
- `cargo test -p instinct-core --test bank traps` → especies disponibles tras la carga.
- `cargo run -p instinct-cli -- check examples/traps/creatures/<trampa>.ins` → exit 1 con `archivo:línea: mensaje` (repetir por cada trampa).
- `cargo run -p instinct-cli -- run --root examples/traps --map Arena --seed 1 --ticks 2 --place Valid@4,4` → exit 0, criatura válida simulando y diagnósticos de las trampas en stderr.
- `cargo test -p instinct-cli --test cli_bank` y `cargo test -p instinct-core --test bank` → exit 0 (todos los scenarios del README).
- `cargo test --workspace` → sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- **Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin cambios en `crates/*/src/` + README completo con Desviaciones.

## Si te bloqueas

Si el rugido no es visible en la terminal sola (sin oyente que lo diga), el
resultado del README se limita al evento de rugido y la percepción la fija la
prueba del núcleo. Si una clase de error del enunciado no produce diagnóstico en
la referencia, es una desviación real: documéntala en Desviaciones con el archivo
trampa que la evidencia y repórtala a F7; no la implementes aquí. Decisiones que
NO son tuyas: cambiar el rugido, el dicho o la rotación de rugidos; cambiar el
cargador o el constructor del mundo; alterar escenarios de otros issues ni tocar
`crates/*/src/`.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: `feat/f6-06-rugidos-say-trampas-cierre` (base `develop`, ya incluye F6-01–F6-05).

Archivos creados (datos y pruebas; sin tocar `crates/*/src/` ni Cargo.toml):
- `examples/creatures/{shouter,listener,singer}.ins` — rugente, oyente y cantante (S42-S46).
- `examples/traps/terrains/grass.te`, `examples/traps/objects/rock.ob`, `examples/traps/maps/arena.map`, `examples/traps/creatures/valid.ins` — raíz de trampas con recursos válidos (S47-S52).
- `examples/traps/creatures/` — 11 archivos `.ins` trampa, uno por clase de error de compilación 2.8 (S47-S51).
- `examples/traps/maps/{unknown_char,ragged}.map` — carácter desconocido y fila dispareja (S53-S54).
- `examples/traps/terrains/bad.te`, `examples/traps/objects/bad.ob` — clave inválida de cada tipo (S55).
- `crates/instinct-cli/tests/cli_bank/{roars,traps,traps_maps}.rs` — scénarios desde terminal (+ `helpers.rs::traps_dir`; `mod.rs` declara los módulos).
- `crates/instinct-core/tests/bank/{roars,traps}.rs` — disponibilidad y alcance internos (+ `mod.rs` declara los módulos).
- `examples/README.md` — escenarios S42-S55, sección Desviaciones (caso concreto S53) y comprobación de cierre.

No se modificó `crates/*/src/` ni `Cargo.toml`.

Evidencia AC (todas PASS; comando exacto de verificación):
- AC1 (PASS): `cargo test -p instinct-cli --test cli_bank roars::roar_emits_event` → evento de rugido con su texto en el tick 0 (`... ok`).
- AC2 (PASS): `cargo test -p instinct-cli --test cli_bank roars::listener_reads_last_roar_next_tick` → tick 0 dice `""` y 0; tick 1 dice el texto y conteo 1 (`... ok`).
- AC3 (PASS): `cargo test -p instinct-core --test bank roars::own_roar_is_not_heard` → la criatura no cuenta ni lee su propio rugido (`... ok`).
- AC4 (PASS): `cargo test -p instinct-core --test bank roars::roar_outside_vision_is_not_heard` → Listener@8,8 a distancia 6 > visión 5; sigue diciendo texto vacío y cero (`... ok`).
- AC5 (PASS): `cargo test -p instinct-cli --test cli_bank roars::say_does_not_consume_turn` → la cantante dice dos textos y se mueve a (3,2) en el mismo tick (`... ok`).
- AC6 (PASS): `cargo test -p instinct-cli --test cli_bank traps::check_rejects_every_trap_with_line` → las 11 trampas: exit 1 y diagnóstico `archivo:línea: mensaje` en stderr (`... ok`).
- AC7 (PASS): `cargo test -p instinct-cli --test cli_bank traps::check_accepts_valid_creature` → `check` del válido: `OK`, exit 0 (`... ok`).
- AC8 (PASS): `cargo test -p instinct-cli --test cli_bank traps::load_traps_root_keeps_valid_creature` → `run` sobre `examples/traps`: exit 0, `Valid` disponible y simulando, diagnósticos de trampas por stderr (`... ok`).
- AC9 (PASS): `cargo test -p instinct-cli --test cli_bank traps_maps::unknown_char_reports_and_loads` → exit 0, diagnóstico que nombra el carácter desconocido y la línea de su fila (`... ok`).
- AC10 (PASS): `cargo test -p instinct-cli --test cli_bank traps_maps::ragged_row_reports_line` → exit 0, diagnóstico con la línea de la fila dispareja (`... ok`).
- AC11 (PASS): sección Desviaciones del README documenta la desviación real S53 (carácter desconocido: el enunciado 3.3 dice «se rechaza»; la referencia conserva el mapa, casilla = primer terreno, diagnóstico duplicado al cargar y al construir el mundo) con caso concreto en `examples/traps/maps/unknown_char.map` (lectura de la sección contrastada con F6-01 a F6-06).
- AC12 (PASS): puertas — `cargo test -p instinct-cli --test cli_bank` 35/35 exit 0; `cargo test -p instinct-core --test bank` 41/41 exit 0 (incluye el nuevo `roar_tick_0_is_readable_next_tick`); `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 5 --place Sprinter@2,2` exit 0; `check` por trampa exit 1; `run` raíz traps exit 0; `cargo test --workspace` rojo solo por determinismo pre-existente de `instinct-gui` (reportado, ajeno a F6-06, no bloquea); `cargo clippy --workspace --all-targets -- -D warnings` rojo por 3 lints pre-existentes en `crates/instinct-core/src/` (assignment.rs, visitor.rs, grid.rs; Rust 1.100.0, ajenos a F6-06, no bloquean); `cargo fmt --all --check` rojo solo por `crates/instinct-gui/src/ui/mod.rs` (formato pre-existente, ajeno a F6-06, no se toca).

`cargo test --workspace`: sin regresiones de F6-06; rojo únicamente en la determinismo pre-existente de `instinct-gui`, reportado y fuera de esta fase.
