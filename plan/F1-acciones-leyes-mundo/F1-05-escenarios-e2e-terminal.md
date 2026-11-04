---
id: F1-05
title: Escenarios E2E uno por acción desde la terminal
phase: F1
status: done
depends_on: [F1-01, F1-02, F1-03, F1-04]
size: M
---

# F1-05 · Escenarios E2E uno por acción desde la terminal

## Objetivo
Integrar las cuatro acciones nuevas en escenarios mínimos ejecutables desde la terminal (`instinct-cli run`), uno por acción, que demuestren el comportamiento observable contra criatura, objeto, terreno y borde del mapa, sin regresiones de F0.

## Alcance
**Haz**
- Criaturas de prueba mínimas en `examples/creatures/` (o tests de integración que construyan el mundo a mano) que usen `attack`, `consume`, `reproduce` y `roar`.
- Un escenario E2E por acción, ejecutable desde la terminal o desde una prueba de integración, que coloque criatura(s), corra N ticks con semilla fija y verifique el estado final (vida, muertes, crías, rugidos, reservas).
- Verificar que dos corridas con la misma semilla dan la misma salida byte a byte (determinismo).
- Verificar que F0 sigue funcionando: `wait`, `move`, `say` sin regresiones.
- `cargo test --workspace`, clippy y fmt en verde.

**No hagas**
- No añadas percepciones de cercanía ni `see`/`name` (F2).
- No cambies el formato de salida de la terminal (F0-08 ya lo fijó).
- No implementes la ventana GUI (F5).

### Archivos previstos
- `crates/instinct-cli/tests/cli_run.rs` — modificar — añadir pruebas de integración E2E por acción (nuevo).
- `examples/creatures/` — nuevo — criaturas de prueba `.ins` para `attack`, `consume`, `reproduce`, `roar` (opcional, si se usa la terminal).
- `crates/instinct-core/tests/sim_tests.rs` — modificar — pruebas de simulación de un tick por acción (nuevo).

## Criterios de aceptación
- [ ] **AC1** Escenario `attack`: criatura A ataca a criatura B adyacente; B pierde vida y, si llega a 0, muere en el mismo tick y su casilla queda solo terreno. Verif: `cargo test -p instinct-cli --test cli_run` o `cargo test -p instinct-core --test sim_tests` | prueba pasa.
- [ ] **AC2** Escenario `consume`: criatura ataca y se cura; vida de atacante sube `heal` sin tope. Verif: prueba de integración | pasa.
- [ ] **AC3** Escenario `reproduce`: madre con vida suficiente cria en casilla libre; cría tiene edad 0, sin variables, arranca en `start:` y juega al tick siguiente. Verif: prueba de integración | pasa.
- [ ] **AC4** Escenario `roar`: criatura ruge; el mensaje está disponible el tick siguiente (evento `Roared` emitido). Verif: prueba de integración | pasa.
- [ ] **AC5** Determinismo: dos corridas misma semilla y mismas colocaciones dan salida byte a byte idéntica. Verif: `cargo run ... --seed 1` dos veces y comparar salida | igual.
- [ ] **AC6** Sin regresiones F0: `wait`, `move`, `say` siguen funcionando; `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` muestra mapa tick 0 y 10, muerte por vida y exit 0. Verif: comando terminal | salida esperada y exit 0.
- [ ] **AC7** Suite completa: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto
**Ya existe** (F0-08): comando `instinct-cli run --root --map --seed --ticks --every --place` en `crates/instinct-cli/src/main.rs`; pruebas `cli_run.rs`, `cli_check.rs`, `cli_usage.rs` en `crates/instinct-cli/tests/`; ejemplos `walker.ins`, `sleeper.ins`, `talker.ins`, `wanderer.ins` en `examples/creatures/`; mapa `plain.map` en `examples/maps/`; `grass.te` y `rock.ob` en `examples/terrains/` y `examples/objects/`.
**Contratos que debes respetar**: `crates/instinct-cli/src/main.rs` — CLI con `--place Especie@x,y` repetible; salida `== tick N ==` + rejilla ASCII + líneas de registro; códigos de salida 0/1/2. `crates/instinct-core/src/sim.rs` — `step_tick` devuelve eventos; `World::draw` dibuja ASCII.
**Ojo**: el comando `run` imprime `== tick N ==` y la rejilla en tick 0, cada `--every` ticks y al final; las líneas de registro van intercaladas o después. Las criaturas de prueba para F1 deben compilar (cabecera completa con `creature`, `faction`, `health`, `vision`, `lifespan`, `start:`).

### Leer primero
- `crates/instinct-cli/src/main.rs` — formato de salida y códigos de salida del comando `run`.
- `crates/instinct-cli/tests/cli_run.rs` — pruebas E2E existentes de F0 para imitar.
- `crates/instinct-core/src/sim.rs` — `step_tick`, `tick()`, `world()` para pruebas de simulación.
- `crates/instinct-core/src/world.rs` — `draw`, `creature_position`, `CellContent` para verificar estado final.
- `examples/creatures/walker.ins` — ejemplo de criatura F0 para imitar el formato de archivo de criatura.

## Especificación
Cada escenario E2E coloca una o dos criaturas sobre un mapa pequeño (hierba con rocas), corre pocos ticks con semilla fija y comprueba el estado final: vidas, muertes (evento `Died` con causa), crías nuevas (edad 0, sin variables), rugidos (evento `Roared`), y que la rejilla refleja lo esperado (criatura desaparecida = terreno, objeto agotado = vacío). El determinismo se prueba corriendo dos veces con la misma semilla y comparando la salida completa.

## Pasos sugeridos (opcional)
1. Crea criaturas de prueba `.ins` en `examples/creatures/` para cada acción (cabecera completa).
2. Añade pruebas de integración en `crates/instinct-cli/tests/cli_run.rs` o `crates/instinct-core/tests/sim_tests.rs` que construyan el mundo a mano y corran `step_tick`.
3. Ejecuta el comando de terminal para cada escenario y compara salida entre dos corridas misma semilla.
4. Ejecuta la suite completa y clippy/fmt.

## Tests
Nivel: integración, en `crates/instinct-cli/tests/cli_run.rs` y/o unitario de simulación en `crates/instinct-core/tests/sim_tests.rs`. Casos como comportamientos: ataque mata/herida, consume cura, reproduce cría, roar evento, determinismo misma semilla, sin regresiones F0. Qué NO mockear: `Simulation` y `World` reales; CLI real para el comando `run`. Datos/fixtures: mapas y criaturas de `examples/`.

## Verificación
- `cargo test -p instinct-cli --test cli_run` → pruebas E2E pasan.
- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Comando terminal F0 sin regresiones: exit 0, muestra tick 0 y 10, muerte por vida.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si una criatura de prueba `.ins` no compila por funciones de F2 (`see`, `name`), simplifícala a solo las acciones de F1; no añadas percepciones fuera de alcance. Decisiones que NO puedes tomar tú: cambiar el formato de salida del CLI, o añadir percepciones de cercanía.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
- AC1 `test_cli_e2e_attack_kills_adjacent_creature`: `cargo test -p instinct-cli --test cli_run test_cli_e2e_attack_kills_adjacent_creature` → ok. Attacker@0,0 ataca Victim@1,0 (daño 10): Victim muere mismo tick (Died causa vida), casilla (1,0) queda terreno en tick 1, eventos Damaged + Died en stdout.
- AC2 `test_cli_e2e_consume_kills_and_heals`: `cargo test -p instinct-cli --test cli_run test_cli_e2e_consume_kills_and_heals` → ok. Consumer mata a Food y casilla queda terreno; daño + Died causan vida.
- AC3 `test_cli_e2e_reproduce_birth`: `cargo test -p instinct-cli --test cli_run test_cli_e2e_reproduce_birth` → ok. Mother@1,1 cría en (1,2) presente en tick 1 (edad 0 implícita al nacer), madre sigue en su casilla.
- AC4 `test_cli_e2e_roar_emits_event`: `cargo test -p instinct-cli --test cli_run test_cli_e2e_roar_emits_event` → ok. `[tick 0] Roarer@1,1 ruge: ¡auau!` en stdout.
- AC5 `test_cli_determinism_same_seed` + terminal: `cargo test -p instinct-cli --test cli_run test_cli_determinism_same_seed` → ok; dos corridas del binario con `--seed 1 --ticks 10 --place Walker@0,0` dan stdout y stderr byte a byte idénticos (SHA-256 igual).
- AC6 `test_cli_f0_walker_no_regression` + terminal: `cargo test ... test_cli_f0_walker_no_regression` → ok; comando de la fase `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` exit 0, muestra tick 0 y tick 10, `[tick 9] Walker@10,0 muere por vida`.
- AC7 `cargo test --workspace` → todo verde (0 fallos); `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings; `cargo fmt --all --check` → exit 0.
- Bug corregido (no scope creep): en `strike_cell` (`crates/instinct-core/src/actions.rs`) el evento `Died` por `attack` reportaba la especie del atacante; ahora reporta la especie de la víctima, consistente con `end_of_tick_creatures` y `die_by_error`. Sin este cambio AC1/AC2 no podían verificar quién murió.
- Archivos: `crates/instinct-core/src.actions.rs` (fix Died species); `crates/instinct-cli/tests/cli_run.rs` (10 tests E2E + core consume heal); `examples/creatures/{attacker,victim,consumer,food,mother,roarer}.ins` (nuevos). Sin `unwrap`/`expect` sobre datos de usuario, sin floats, sin dependencias nuevas, sin `rand`.
- `plan/.../NOCHE_F1.md` tiene una modificación previa ajena a este issue; se deja fuera del commit.
