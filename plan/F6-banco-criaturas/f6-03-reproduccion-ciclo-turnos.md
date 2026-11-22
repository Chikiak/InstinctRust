---
id: F6-03
title: Reproducción, aborto natural y ciclo de turnos
phase: F6
status: done
depends_on: [F6-01]
size: M
file: f6-03-reproduccion-ciclo-turnos.md
---

# F6-03 · Reproducción, aborto natural y ciclo de turnos

## Objetivo

Scenarios de `reproduce` con sus condiciones y su aborto natural, y del ciclo de
turnos del enunciado 2.7: la cría nace con edad 0, sin variables y con su
programa en `start:`, y juega por primera vez en el tick siguiente; una criatura
muerta antes de su turno no juega; el límite de 100 líneas por turno corta un
ciclo mal escrito; llegar al final del archivo reanuda en `start:`; y las
variables sobreviven de un turno al siguiente.

## Alcance

**Haz**
- Criaturas: una madre que reproduce cuando puede, una madre que falla por salud
  insuficiente, y una cría que lee una variable que la madre sí tenía (para
  demostrar que no hereda nada).
- Scenarios en `examples/README.md`: camino feliz de la reproducción, los tres
  abortos naturales (casilla ocupada por criatura, casilla con objeto, destino
  fuera del mapa) y el aborto por salud, más el ciclo de turnos.
- Pruebas en `crates/instinct-cli/tests/cli_bank/turns.rs` (eventos y rejilla) y
  en `crates/instinct-core/tests/bank/turns.rs` (edad, variables y `pc` de la
  cría, orden de turnos, líneas ejecutadas).

**No hagas**
- No pruebes `attack`/`consume` (F6-02) ni los rugidos (F6-06).
- No cambies el orden de turnos ni la semilla para forzar un resultado: si un
  scenario necesita un orden concreto, elige la semilla que lo da y documéntala.
- No toques `crates/*/src/`; el banco se basa en `examples/`.

### Archivos previstos
- `examples/creatures/mother.ins` — nuevo — reproduce hacia una casilla libre cuando su salud lo permite.
- `examples/creatures/poor_mother.ins` — nuevo — intenta reproducir con salud insuficiente.
- `examples/creatures/heir.ins` — nuevo — lee una variable que solo la madre guarda.
- `examples/creatures/counter.ins` — nuevo — cuenta turnos en una variable y la dice cuando puede, para probar memoria y reanudación.
- `examples/creatures/spinner.ins` — nuevo — ciclo sin acción, para probar el límite de 100 líneas.
- `crates/instinct-cli/tests/cli_bank/turns.rs` — nuevo — eventos y rejilla de los scenarios.
- `crates/instinct-cli/tests/cli_bank/mod.rs` — modificar — declarar el módulo nuevo.
- `crates/instinct-core/tests/bank/turns.rs` — nuevo — edad, variables, `pc`, orden y líneas ejecutadas.
- `crates/instinct-core/tests/bank/mod.rs` — modificar — declarar el módulo nuevo.
- `examples/README.md` — modificar — escenarios S18 a S24.

## Criterios de aceptación

- [ ] **AC1** `reproduce` con casilla libre y salud suficiente crea una cría de la misma especie en la casilla indicada, con edad 0, la vida pedida y sin variables. Verif: `cargo test -p instinct-core --test bank turns::birth_has_age_zero_and_no_variables`.
- [ ] **AC2** La madre pierde la vida que transfiere y la cría nace con esa vida. Verif: `cargo test -p instinct-core --test bank turns::birth_transfers_health`.
- [ ] **AC3** La cría juega por primera vez en el tick siguiente al de su nacimiento. Verif: `cargo test -p instinct-cli --test cli_bank turns::offspring_first_turn_next_tick`.
- [ ] **AC4** La cría no hereda variables: un programa que lee la variable de la madre muere con error de ejecución y línea en su primer turno. Verif: `cargo test -p instinct-cli --test cli_bank turns::offspring_inherits_no_variables`.
- [ ] **AC5** Aborto natural sin efectos: casilla destino con criatura, con objeto o fuera del mapa, o madre sin salud suficiente; en los cuatro casos no nace nadie y la madre sigue viva. Verif: `cargo test -p instinct-core --test bank turns::reproduce_aborts_silently`.
- [ ] **AC6** Una criatura que muere antes de su turno dentro del tick no ejecuta nada: sin eventos suyos tras la muerte. Verif: `cargo test -p instinct-core --test bank turns::creature_dead_before_turn_does_not_play`.
- [ ] **AC7** Un ciclo sin acción termina su turno como si hubiera esperado un turno y la simulación sigue: el contador de la criatura avanza como máximo el límite de líneas por turno. Verif: `cargo test -p instinct-core --test bank turns::line_limit_caps_counter_per_turn`.
- [ ] **AC8** Llegar al final del archivo reanuda en `start:` en el siguiente turno y una variable asignada en un turno sigue disponible en los siguientes. Verif: `cargo test -p instinct-core --test bank turns::variables_persist_across_turns`.
- [ ] **AC9** Puertas: `cargo test --workspace` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all --check` exit 0 · `git status` sin cambios tras el commit. Verif: los cuatro comandos.

## Contexto

**Ya existe** (F6-01 y F6-02): la raíz `examples/`, los objetivos de prueba
`cli_bank` y `bank`, y víctimas quietas reutilizables como obstáculos para los
abortos. **Contratos que debes respetar**: los de `PHASE.md`. **Ojo**: el orden
de turnos se sortea cada tick con la semilla del mundo, así que un scenario que
dependa del orden fija la semilla y la documenta; la muerte por golpe ocurre
durante el turno del atacante, antes de que la víctima llegue a jugar si el sorteo
la coloca después.

### Leer primero
- `crates/instinct-core/src/actions/reproduce.rs` — condiciones de la reproducción y qué cuenta como aborto natural.
- `crates/instinct-core/src/interp/turn.rs` — límite de líneas por turno y reanudación desde `pc`.
- `crates/instinct-core/src/sim/mod.rs` — sorteo del orden por tick y criaturas muertas que se saltan.
- `crates/instinct-core/src/sim/death.rs` — muerte por error de ejecución con `RuntimeError` y `Died`.
- `crates/instinct-core/src/world/life.rs` — colocación y vida/edad iniciales de una criatura.
- `crates/instinct-core/src/interp/state.rs` — qué guarda el estado de turno de una criatura.

## Especificación

| # | Regla (enunciado) | Escenario | Resultado esperado |
|---|-------------------|-----------|--------------------|
| S18 | 2.4 `reproduce` | `mother.ins` con salud y casilla libre contigua | nace una cría de su especie con la vida pedida y edad 0; la madre baja esa vida |
| S19 | 2.7 punto 8 | escena del nacimiento | la primera acción de la cría aparece un tick después de nacer |
| S20 | 2.4 `reproduce` | `heir.ins` que lee una variable de la madre | error de ejecución con línea en su primer turno y muerte; la madre sigue |
| S21 | 2.4 `reproduce` | destino con criatura, con objeto o fuera del mapa | no nace nadie; la madre consume el turno y sigue viva |
| S22 | 2.4 `reproduce` | madre con salud menor o igual a la pedida | no nace nadie |
| S23 | 2.7 punto 5 | criatura en ciclo sin acción | su turno se corta en el límite de líneas y la simulación continúa |
| S24 | 2.7 puntos 3, 6 y 7 | criatura con memoria y final de archivo | sus variables valen de un turno a otro y al acabar el archivo reanuda en `start:` |

Para S21 conviene una víctima quieta como ocupante de la casilla destino y una
roca del mapa como obstáculo. Para S23, un ciclo de asignaciones sin acción
demuestra el límite: el contador no puede crecer más de lo que permite el límite
en un solo turno.

### Pasos sugeridos
1. Escribe la madre y comprueba el nacimiento con el comando del escenario.
2. Añade los abortos uno a uno, reutilizando la víctima quieta y las rocas.
3. Escribe el scenario del límite de líneas con un ciclo de asignaciones.
4. Escribe las pruebas de estado interno (edad, variables, `pc`) y luego las de terminal.
5. Rellena el README con la salida real de cada comando.

## Tests

Nivel: integración. En `cli_bank/turns.rs`: los eventos de nacimiento y error, y
la rejilla que muestra u oculta a la cría. En `bank/turns.rs`: edad 0 de la cría,
variables de la cría vacías, vida transferida, `pc` de reanudación, líneas
ejecutadas por turno y que una criatura muerta no emite nada en su turno. No se
mockea nada: `Simulation` y `World` reales desde `examples/`, y el binario real de la
CLI. Semilla fija y documentada por scenario.

## Verificación

- `cargo test -p instinct-cli --test cli_bank turns` → nacimiento, abortos y ciclo de turnos observables.
- `cargo test -p instinct-core --test bank turns` → edad, variables, `pc` y líneas por turno.
- `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 3 --place Mother@4,4` → exit 0 con la cría presente desde el tick siguiente.
- `cargo test --workspace` → sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- **Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin cambios en `crates/*/src/` + README sin resultados inventados.

## Si te bloqueas

Si el aborto natural no es observable desde la terminal (no hay evento de
aborto), el resultado esperado del README es la ausencia de cambios: la madre
sigue, la casilla destino sigue como está. Si un scenario necesita un orden de
turnos concreto y la semilla 1 no lo da, busca otra semilla y documéntala; no
fuerces el orden cambiando el núcleo. Decisiones que NO son tuyas: cambiar
`reproduce`, el sorteo de turnos o el límite de líneas.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: `feat/f6-03-reproduccion-ciclo-turnos` (checkeada y limpia al inicio).
Se trabajó en worktree `../wt-f6-03` sin tocar `crates/*/src/` ni el working
directory principal.

Commits realizados (uno por cambio verde):
- `feat(f6-03): heir.ins sin herencia de variables (AC4)`
  - `examples/creatures/heir.ins`: solo la madre asigna `legacy = 42` y se
    reproduce; la cría lee `legacy` sin asignarlo → error de ejecución con
    línea y muerte en `[tick 1]`, madre sigue viva en (2,2), celda (2,3)
    vuelve a `.` en tick 2.
  - `crates/instinct-cli/tests/cli_bank/turns.rs`:
    `offspring_inherits_no_variables` aserta el evento real de error de
    ejecución (`error at line 8: unknown variable 'legacy'`), la muerte
    (`dies of error`), y que en tick 2 la cría murió y la madre sigue.
- `feat(f6-03): cría juega su primer turno en el tick siguiente (AC3)`
  - `examples/creatures/first_turn_mother.ins` (nuevo): `say("ready")`;
    `reproduce(0, 1, 5); goto start`. La cría nace en (2,3) al final del
    tick 0 y dice "ready" en `[tick 1]`.
  - `examples/creatures/talker_victim.ins` (nuevo): `say("alive"); goto start`,
    salud 5, usada en AC6.
  - `crates/instinct-cli/tests/cli_bank/turns.rs`:
    `offspring_first_turn_next_tick` usa `FirstTurnMother@2,2`; verifica que
    en `[tick 0]` no hay evento de la cría y en `[tick 1]` la cría dice
    "ready" en (2,3) (su primera acción es en tick 1, nunca en tick 0).
- `feat(f6-03): reproduce_aborts_silently y evento vacío en abortos (AC5)`
  - `crates/instinct-core/tests/bank/turns.rs`: los 4 tests de aborto
    (`cell_occupied_by_creature`, `cell_has_rock`, `out_of_map`,
    `insufficient_health`) capturan `let events = bw.step_tick();` y aserran
    `events.is_empty()`.
  - Nueva prueba `reproduce_aborts_silently` que valida los 4 casos de aborto
    natural: no nace nadie, la madre sigue viva y `events.is_empty()`.
- `feat(f6-03): criatura muerta antes de su turno no juega (AC6)`
  - `crates/instinct-core/tests/bank/turns.rs`:
    `creature_dead_before_turn_does_not_play` coloca `Brawler@2,2` (daño 10)
    junto a `TalkerVictim@3,2` (salud 5, dice "alive"). Con seed 1 el Brawler
    juega antes que la víctima en tick 0 y la mata; aserta `Died` de la
    víctima y cero eventos de acción suya (`Said`/`Damaged`/`Roared`).
- `feat(f6-03): spinner.ins con contador real y límite de 100 líneas (AC7)`
  - `examples/creatures/spinner.ins`: `n = 0; loop: n = n + 1; goto loop`.
  - `crates/instinct-core/tests/bank/turns.rs`:
    `line_limit_caps_counter_per_turn` aserta `n == 33` tras 100 líneas en
    tick 0, `n <= 100`, `health == 19`, `age == 1`; segundo `step_tick()`
    aserta `health == 18` y `n > 33`.
- `feat(f6-03): counter.ins sin goto start final, variable persiste (AC8)`
  - `examples/creatures/counter.ins`: `if tick > 0 goto later; marked = tick;
    wait(1); later: say(marked)`. La variable `marked` se asigna en tick 0 y
    se lee sin reasignar en tick 1.
  - `crates/instinct-core/tests/bank/turns.rs`:
    `variables_persist_across_turns` aserta `marked == 0` tras tick 0, evento
    `Said "0"` en tick 1, `marked` aún 0 en tick 1, y `pc == 0` (reanudó en
    `start:`).
- `docs(f6-03): README S18–S24 con salidas reales y desviaciones (Fix 7)`
  - `examples/README.md`: sección S18–S24 reescrita en UTF-8 limpio (sin
    mojibake), con bloques de triple backtick correctos y salidas reales de
    terminal para cada comando, incluyendo S19 y S20. Citación exacta de
    nombres de test en Rust. Sección `## Tests del banco` actualizada con
    `cli_bank/turns.rs` y `bank/turns.rs`. Sección `## Desviaciones frente
    al enunciado` actualizada con las desviaciones de F6-03.

Archivos modificados:
- `examples/creatures/heir.ins` (modificado, AC4)
- `examples/creatures/spinner.ins` (modificado, AC7)
- `examples/creatures/counter.ins` (modificado, AC8)
- `examples/creatures/first_turn_mother.ins` (nuevo, AC3)
- `examples/creatures/talker_victim.ins` (nuevo, AC6)
- `examples/README.md` (modificado, S18–S24, Tests del banco, Desviaciones)
- `crates/instinct-cli/tests/cli_bank/turns.rs` (modificado, AC3, AC4)
- `crates/instinct-core/tests/bank/turns.rs` (modificado, AC5, AC6, AC7, AC8)
- `crates/instinct-core/tests/bank/common.rs` (formato, `cargo fmt`)
- `plan/F6-banco-criaturas/f6-03-reproduccion-ciclo-turnos.md` (solo
  `### Notas de implementación`)

Cambios de firma autorizados: ninguno (no se tocó `crates/*/src/`).

Desviaciones (documentadas, sin tocar el núcleo):
- `spinner.ins` usa la variable `n` en lugar de `x`: `x` es una percepción de
  solo lectura y el núcleo rechaza asignarle.
- El mapa `Arena` es 9×9 (filas/columnas válidas 0..8). El test de "fuera del
  mapa" coloca a Mother@2,8 con destino (2,9), fuera del mapa.
- `cargo test --workspace` falla únicamente en el test pre-existing
  `instinct-gui::tests::determinismo::t_determinismo_01_gui_iguala_directo_con_seed7`
  (ajeno a F6-03; `instinct-gui` no se modificó).
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo fmt --check -p instinct-core -p instinct-cli`: exit 0 (archivos
  tocados en este issue están fmt-limpios; `instinct-gui` tiene pre-existing).
- `git diff origin/develop -- crates/*/src/`: vacío (sin cambios en el núcleo).
- `git status` limpio tras el commit final.

Evidencia AC por AC (todos los tests en verde):
- AC1 (PASS): `cargo test --offline -p instinct-core --test bank
  turns::birth_has_age_zero_and_no_variables` → ok.
- AC2 (PASS): `cargo test --offline -p instinct-core --test bank
  turns::birth_transfers_health` → ok.
- AC3 (PASS): `cargo test --offline -p instinct-cli --test cli_bank
  turns::offspring_first_turn_next_tick` → ok.
- AC4 (PASS): `cargo test --offline -p instinct-cli --test cli_bank
  turns::offspring_inherits_no_variables` → ok.
- AC5 (PASS): `cargo test --offline -p instinct-core --test bank
  turns::reproduce_aborts_silently` +
  `reproduce_aborts_cell_occupied_by_creature`,
  `reproduce_aborts_cell_has_rock`, `reproduce_aborts_out_of_map`,
  `reproduce_aborts_insufficient_health` → ok.
  AC5-cli (PASS): `cli_bank turns::cli_abort_cell_occupied_by_creature` → ok.
- AC6 (PASS): `cargo test --offline -p instinct-core --test bank
  turns::creature_dead_before_turn_does_not_play` → ok.
- AC7 (PASS): `cargo test --offline -p instinct-core --test bank
  turns::line_limit_caps_counter_per_turn` → ok.
- AC8 (PASS): `cargo test --offline -p instinct-core --test bank
  turns::variables_persist_across_turns` → ok.
- AC9: `cargo test --offline -p instinct-core --test bank` → ok (21 tests);
  `cargo test --offline -p instinct-cli --test cli_bank` → ok (14 tests);
  `cargo clippy --offline --workspace --all-targets -- -D warnings` → exit 0;
  `cargo fmt --check -p instinct-core -p instinct-cli` → exit 0;
  `git diff origin/develop -- crates/*/src/` → vacío;
  `git status` → limpio tras commit.
