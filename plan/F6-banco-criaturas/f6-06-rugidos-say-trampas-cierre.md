---
id: F6-06
title: Rugidos y say, trampas de compilación, mapas inválidos y cierre
phase: F6
status: todo
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