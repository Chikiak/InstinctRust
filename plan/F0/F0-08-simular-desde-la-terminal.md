---
id: F0-08
title: Simular desde la terminal
phase: F0
status: todo
depends_on: [F0-07]
size: M
---

# F0-08 · Simular desde la terminal

## Objetivo
Que `instinct-cli run` cargue los recursos, coloque criaturas, simule N ticks con una semilla y muestre mapa y registro. Cierra el camino feliz de la fase y es lo que se usa para probar criaturas a mano.

## Alcance
**Haz**
- Opciones `--seed`, `--every` y `--place` (repetible) del subcomando `run`, y el avance de `--ticks`.
- Salida de mapas y líneas de registro según la tabla de comandos de la fase; diagnósticos por stderr y códigos de salida.
- Criaturas de ejemplo `sleeper.ins`, `talker.ins` y `wanderer.ins`.

**No hagas**
- Ventana ni colores; ninguna dependencia externa para leer argumentos.
- Archivos de colocación (solo `--place`).

### Archivos previstos
- `crates/instinct-cli/src/main.rs` — modificar — opciones y bucle de simulación.
- `crates/instinct-cli/tests/cli_simulate.rs` — nuevo.
- `examples/creatures/sleeper.ins`, `talker.ins`, `wanderer.ins` — nuevos.

## Criterios de aceptación
- [ ] **AC1** `run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` muestra el mapa del tick 0 con `W` en la columna 0 de la fila 0 y el del tick 10 sin criatura, la línea `[tick 9] Walker@(10,0) muere por vida` y sale con 0. Verif: `test_cli_walker_dies_of_health`
- [ ] **AC2** Con `--place Walker@0,1` y 5 ticks, el `W` queda en la columna 2 de la fila 1 y no entra en la roca de la columna 3. Verif: `test_cli_walker_blocked_by_rock`
- [ ] **AC3** `Sleeper` colocado y 10 ticks emite su muerte `por edad` en el tick 4 (esperanza 5, 0-based). Verif: `test_cli_creature_dies_of_age`
- [ ] **AC4** `Talker` emite una sola línea `[tick 0] Talker@(2,2) dice: hola` y no repite el saludo. Verif: `test_cli_say_is_logged_once`
- [ ] **AC5** Con `--every 5 --ticks 10` se imprimen los mapas de los ticks 0, 5 y 10 y ningún otro. Verif: `test_cli_every_option`
- [ ] **AC6** Una colocación sobre la roca, fuera del mapa, con especie desconocida o con formato mal escrito escribe un diagnóstico por stderr, no simula y sale con 1. Verif: `test_cli_rejects_bad_placement`
- [ ] **AC7** La misma orden con la misma semilla da la misma salida byte a byte; con `Wanderer` y dos semillas distintas la salida difiere. Verif: `test_cli_reproducible_output`
- [ ] **AC8** Suite completa, clippy y fmt en verde, y el comando de salida de la fase funciona. Verif: los comandos de Verificación

## Contexto
**Ya existe**: `run` con `--root`, `--map` y `--ticks` que imprime el tick 0 (F0-02) en `crates/instinct-cli/src/main.rs`; `check` (F0-06); `Simulation` (F0-07) en `crates/instinct-core/src/sim.rs`; ejemplo `walker.ins`.
**Contratos que debes respetar**: tabla "Comandos de la terminal", "Eventos" y "Recursos de ejemplo" en `../PHASE.md#contratos-compartidos`.
**Ojo**: el número de un evento es el valor de `tick` durante el cual ocurrió, de modo que la muerte por vida del Walker (vida 10) sale en el tick 9 aunque el mapa final se rotule tick 10. Los ejemplos: `sleeper.ins` vida 100 y esperanza 5, solo espera; `talker.ins` dice "hola" si `tick` vale 0 y luego espera; `wanderer.ins` calcula dx y dy con `random` en el rango de -1 a 1 y se mueve de a una casilla.
**Rust que se practica**: lectura manual de argumentos, `Display`, separar parseo de argumentos de lógica, pruebas de integración que capturan la salida del binario.

### Leer primero
- `crates/instinct-cli/src/main.rs` — organización de subcomandos.
- `crates/instinct-core/src/sim.rs` — API para crear y avanzar la simulación.
- `crates/instinct-core/src/event.rs` — qué datos trae cada evento.
- `examples/creatures/walker.ins` — estilo de un `.ins` válido.

## Especificación
- `--place` tiene la forma especie, arroba, x, coma, y; la especie debe existir en los recursos cargados.
- Salida de cada mapa: línea `== tick N ==` y las filas. Las líneas de registro salen después del mapa del tick en que ocurrieron, en orden de evento.
- El mapa final se imprime aunque no coincida con `--every`.
- Un fallo de carga de una criatura no pedida con `--place` se informa por stderr pero no impide la simulación; si es la pedida, sale con 1.

## Tests
Integración contra el binario real sobre `examples/`; comprueba salidas con fragmentos exactos de líneas (no la pantalla completa salvo en AC7). Escribe primero AC1 y AC5.

## Verificación
- `cargo test -p instinct-cli` → esperado: AC1 a AC7 en verde.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` → esperado: exit 0.
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` → esperado: el resultado descrito en AC1.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si la API de `Simulation` no basta para imprimir los mapas, repórtalo en lugar de acceder a campos internos del mundo. No decidas tú el formato de salida: está en la tabla de la fase.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Cierra la fase: revisa que `docs/ambiguedades.md` recoja todas las decisiones tomadas y anota en el Registro de ejecución de `PHASE.md` el tamaño real de cada issue.
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
