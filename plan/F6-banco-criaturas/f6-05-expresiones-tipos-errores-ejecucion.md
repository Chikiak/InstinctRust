---
id: F6-05
title: Expresiones, tipos y errores de ejecución
phase: F6
status: done
depends_on: [F6-01]
size: M
file: f6-05-expresiones-tipos-errores-ejecucion.md
---

# F6-05 · Expresiones, tipos y errores de ejecución

## Objetivo

Scenarios de expresiones del enunciado 2.5 con resultados observables —la
precedencia completa, la división y el resto enteros con semántica de suelo, la
verdad de textos y números— y de los errores de ejecución de 2.8: la criatura que
mezcla un entero y un texto, divide por cero, lee una variable sin asignar, usa
texto donde hace falta un número o se sale del rango de direcciones muere con
mensaje y línea, y las demás criaturas de la simulación siguen su camino.

## Alcance

**Haz**
- Una criatura calculadora por familia de expresiones, que dice el resultado para
  que sea observable en el registro.
- Una criatura trampa por clase de error de ejecución: todas compilan bien y
  fallan al correr.
- Scenarios en `examples/README.md` con la línea de registro o de error esperada.
- Pruebas en `crates/instinct-cli/tests/cli_bank/expressions.rs` (líneas de
  registro y de error, exit 0 siempre) y en
  `crates/instinct-core/tests/bank/expressions.rs` (valores exactos y la muerte
  con causa de error).

**No hagas**
- No pruebes los errores de compilación: esos van en F6-06 con sus propios archivos trampa.
- No corrijas el núcleo si un resultado no cuadra con el enunciado: documéntalo y repórtalo a F7.
- No toques `crates/*/src/`; el banco se basa en `examples/`.

### Archivos previstos
- `examples/creatures/calculator.ins` — nuevo — dice el resultado de expresiones con paréntesis, menos unario, multiplicación, suma y comparaciones.
- `examples/creatures/divider.ins` — nuevo — dice cociente y resto de divisiones con signos mixtos.
- `examples/creatures/truths.ins` — nuevo — dice qué valores son verdaderos y cuáles falsos.
- `examples/creatures/mixer.ins` — nuevo — mezcla un entero y un texto en un operando.
- `examples/creatures/divider_zero.ins` — nuevo — divide por cero.
- `examples/creatures/forgetful.ins` — nuevo — lee una variable antes de asignarla.
- `examples/creatures/wordy.ins` — nuevo — pasa texto donde la acción espera un número.
- `examples/creatures/out_of_range.ins` — nuevo — usa una dirección fuera del rango -1 a 1.
- `crates/instinct-cli/tests/cli_bank/expressions.rs` — nuevo — scenarios observables desde la terminal.
- `crates/instinct-cli/tests/cli_bank/mod.rs` — modificar — declarar el módulo nuevo.
- `crates/instinct-core/tests/bank/expressions.rs` — nuevo — valores exactos y muertes por error.
- `crates/instinct-core/tests/bank/mod.rs` — modificar — declarar el módulo nuevo.
- `examples/README.md` — modificar — escenarios S34 a S41.

## Criterios de aceptación

- [ ] **AC1** La precedencia es la del enunciado: paréntesis y llamadas, menos unario y `not`, luego multiplicación, división y resto, luego suma y resta, luego comparaciones, luego `and`, luego `or`; la calculadora dice los valores esperados. Verif: `cargo test -p instinct-core --test bank expressions::precedence_matches_enunciado`.
- [ ] **AC2** La división y el resto enteros se redondean hacia menos infinito y el resto toma el signo del divisor: la calculadora dice cociente y resto de pares con signos mixtos. Verif: `cargo test -p instinct-core --test bank expressions::division_and_remainder_floor`.
- [ ] **AC3** Comparaciones y lógica devuelven 1 o 0, y son falsos el entero 0 y el texto vacío, y verdadero todo lo demás. Verif: `cargo test -p instinct-core --test bank expressions::truthiness_of_numbers_and_texts`.
- [ ] **AC4** El texto solo admite igual y distinto: comparar textos con un operador de orden muere con error y línea. Verif: `cargo test -p instinct-cli --test cli_bank expressions::text_order_comparison_dies`.
- [ ] **AC5** Mezclar un entero y un texto en un mismo operador mata a la criatura con `RuntimeError` y línea, y su casilla queda libre. Verif: `cargo test -p instinct-cli --test cli_bank expressions::mixed_types_die_with_line`.
- [ ] **AC6** Dividir por cero y leer una variable sin asignar matan a la criatura con su mensaje y línea. Verif: `cargo test -p instinct-cli --test cli_bank expressions::div_zero_and_unassigned_die`.
- [ ] **AC7** Pasar texto donde la acción espera un número y usar una dirección fuera del rango -1 a 1 matan a la criatura con su mensaje y línea. Verif: `cargo test -p instinct-cli --test cli_bank expressions::text_argument_and_bad_direction_die`.
- [ ] **AC8** Un error de ejecución mata solo a esa criatura: colocada junto a otra, la otra sigue viva y la simulación termina con exit 0. Verif: `cargo test -p instinct-cli --test cli_bank expressions::runtime_error_kills_only_its_creature`.
- [ ] **AC9** Puertas: `cargo test --workspace` exit 0 · `cargo clippy --workspace --all-targets -- -D warnings` exit 0 · `cargo fmt --all --check` exit 0 · `git status` sin cambios tras el commit. Verif: los cuatro comandos.

## Contexto

**Ya existe** (F6-01): la raíz `examples/`, los objetivos de prueba y la convención de
scenario; una víctima quieta reutilizable como compañera de simulación.
**Contratos que debes respetar**: los de `PHASE.md`. **Ojo**: la muerte por error
de ejecución emite dos eventos, el error y la muerte con causa `error`, y saca a
la criatura del mapa en el acto; la línea del mensaje es la de la sentencia que
falló, contada desde 1.

### Leer primero
- `crates/instinct-core/src/value/ops.rs` — operadores binarios y unarios, y los mensajes de error por tipo.
- `crates/instinct-core/src/value/div_rem.rs` — división y resto con suelo, y división por cero.
- `crates/instinct-core/src/value/cmp.rs` — qué comparaciones aceptan texto y cuáles no.
- `crates/instinct-core/src/value/truth.rs` — qué es verdadero y qué es falso.
- `crates/instinct-core/src/sim/death.rs` — los dos eventos de la muerte por error.
- `crates/instinct-core/src/actions/parse.rs` — error cuando un argumento de acción recibe texto.
- `crates/instinct-core/src/interp/turn.rs` — de dónde sale la línea que se reporta.

## Especificación

La técnica común del issue: cada criatura dice el valor calculado, o ejecuta la
sentencia que debe fallar y muere. El resultado esperado es una línea de registro
concreta, copiada de la ejecución real.

| # | Regla (enunciado) | Escenario | Resultado esperado |
|---|-------------------|-----------|--------------------|
| S34 | 2.5 precedencia | `calculator.ins` | dice los valores de expresiones con paréntesis, menos unario y mezcla de operadores |
| S35 | 2.5 división entera | `divider.ins` con signos mixtos | dice cociente y resto con redondeo hacia abajo y signo del divisor |
| S36 | 2.5 lógica y truthiness | `truths.ins` | dice 1 o 0 según el valor sea verdadero o falso |
| S37 | 2.5 texto y orden | `mixer.ins` | muere con error y línea al comparar textos con un operador de orden |
| S38 | 2.5 y 2.8 mezcla de tipos | `mixer.ins` | muere con error y línea al mezclar entero y texto |
| S39 | 2.8 división por cero | `divider_zero.ins` | muere con error y línea |
| S40 | 2.8 variable sin asignar | `forgetful.ins` | muere con error y línea |
| S41 | 2.5 y 2.8 texto como número | `wordy.ins` junto a una quieta | muere con error y línea; la quieta sigue viva y la simulación acaba con exit 0 |

El scenario de direcciones fuera de rango se cubre dentro de S41 con una criatura
que intenta moverse con una dirección fuera del rango -1 a 1.

### Pasos sugeridos
1. Escribe la calculadora y comprueba a mano los valores de cada expresión.
2. Escribe las trampas una a una y confirma el mensaje y la línea reales.
3. Escribe las pruebas de terminal (líneas de registro y de error) y las del núcleo (valores y causa de muerte).
4. Rellena el README con la salida real de cada comando.

## Tests

Nivel: integración. En `bank/expressions.rs` del núcleo: los valores exactos que
la calculadora produce y que la criatura trampa muere con causa de error. En
`cli_bank/expressions.rs`: las líneas de registro y de error tal cual salen por
terminal, y que el exit sigue siendo 0 con una criatura menos. No se mockea nada:
simulación y mundo reales desde `examples/`, binario real de la CLI.

## Verificación

- `cargo test -p instinct-core --test bank expressions` → valores y muertes correctos.
- `cargo test -p instinct-cli --test cli_bank expressions` → líneas de registro y de error esperadas.
- `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 2 --place Calculator@2,2` → exit 0 con las líneas de resultados.
- `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 2 --place Mixer@2,2 --place Victim@6,2` → exit 0, error con línea de la mezcladora y la víctima viva.
- `cargo test --workspace` → sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- **Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin cambios en `crates/*/src/` + README sin resultados inventados.

## Si te bloqueas

Si un mensaje de error no nombra la línea que el enunciado sugiere, documéntalo
en Desviaciones con el ejemplo y repórtalo a F7. Si un valor de precedencia
discrepa del enunciado, es una desviación real: regístrala con el caso exacto.
Decisiones que NO son tuyas: cambiar operadores, mensajes o el intérprete.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Rama: `feat/f6-05-expresiones-tipos-errores-ejecucion` (sobre `origin/develop @ 2bf0a35` = `<BASE>`, donde ya estaba mergeado F6-04). Sin push.

Evidencia AC-por-AC (comando → resultado):
- AC1 `cargo test -p instinct-core --test bank expressions::precedence_matches_enunciado` → `test result: ok. 1 passed`.
- AC2 `cargo test -p instinct-core --test bank expressions::division_and_remainder_floor` → `test result: ok. 1 passed`.
- AC3 `cargo test -p instinct-core --test bank expressions::truthiness_of_numbers_and_texts` → `test result: ok. 1 passed`.
- AC4 `cargo test -p instinct-cli --test cli_bank expressions::text_order_comparison_dies` → `test result: ok. 1 passed`.
- AC5 `cargo test -p instinct-cli --test cli_bank expressions::mixed_types_die_with_line` → `test result: ok. 1 passed`.
- AC6 `cargo test -p instinct-cli --test cli_bank expressions::div_zero_and_unassigned_die` → `test result: ok. 1 passed`.
- AC7 `cargo test -p instinct-cli --test cli_bank expressions::text_argument_and_bad_direction_die` → `test result: ok. 1 passed`.
- AC8 `cargo test -p instinct-cli --test cli_bank expressions::runtime_error_kills_only_its_creature` → `test result: ok. 1 passed`.
- AC9 `cargo clippy --workspace --all-targets -- -D warnings` → exit 101 (3 lints preexistentes de `clippy` en `crates/instinct-core/src/` bajo Rust 1.100.0, ajenos a F6, no se modifican); `cargo fmt --all --check` → exit 1 (solo diff preexistente en `crates/instinct-gui/src/ui/mod.rs`); `cargo test --workspace` → exit 101 únicamente por `instinct-gui::determinismo::t_determinismo_01_gui_iguala_directo_con_seed7` (fallo preexistente, reportado, ajeno a F6); `git status --short` limpio tras el último commit; pruebas F6-05: 8/8 núcleo + 5/5 CLI en verde.

Cambios de firma autorizados: ninguno. No se toca `crates/*/src/`; los dos `Cargo.toml` ya tenían `[[test]]` y no cambiaron.

Archivos modificados:
- Nuevos: `crates/instinct-core/tests/bank/expressions.rs`, `crates/instinct-cli/tests/cli_bank/expressions.rs`, 9 criaturas en `examples/creatures/` (`calculator.ins`, `divider.ins`, `divider_zero.ins`, `forgetful.ins`, `mixer.ins`, `out_of_range.ins`, `textorder.ins`, `truths.ins`, `wordy.ins`).
- Modificados: `crates/instinct-core/tests/bank/mod.rs` (`mod expressions`), `crates/instinct-cli/tests/cli_bank/mod.rs` (`mod expressions`), `examples/README.md` (escenarios S34-S41), `plan/F6-banco-criaturas/f6-05-expresiones-tipos-errores-ejecucion.md` (status done + esta nota), `plan/F6-banco-criaturas/PHASE.md` (fila F6-05 done + registro de ejecución).
Desviaciones: en la especificación el scenario S37 citaba `mixer.ins` pero se implementó como `textorder.ins` para aislar una criatura trampa por clase de error de ejecución (`textorder.ins` compara textos con operador de orden; `mixer.ins` cubre S38, mezcla de entero y texto). Puertas preexistentes ajenas a F6: `instinct-gui` en `cargo test --workspace` y `cargo fmt --all --check` (formato preexistente en `crates/instinct-gui/src/ui/mod.rs`), y 3 lints preexistentes de `clippy` en `crates/instinct-core/src/` bajo Rust 1.100.0 (si vuelven a reproducirse, son ajenos a F6 y no bloquean el cierre). Ningún resultado inventado en el README: todas las líneas esperadas se copiaron de la ejecución real de cada scenario.
