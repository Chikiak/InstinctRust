---
id: F2-08
title: CLI: validación de argumentos, separación de flujos y pruebas
phase: F2
status: done
depends_on: [F2-03, F2-07]
size: L
---

# F2-08 · CLI: validación de argumentos, separación de flujos y pruebas

## Objetivo

Cerrar los problemas C1, C2, C3, C5 y C4 de la CLI: que los flags desconocidos se rechacen en
lugar de ignorarse, que `--ticks` y `--root` se validen antes de usarse, que la salida de simulación
y los diagnósticos vayan por flujos separados, y que cada caso tenga prueba.

## Alcance

**Haz**

- **Subcomandos como objetos comando (C1)** en `crates/instinct-cli/src/main.rs`: `run`, `check` y
  la ayuda son tipos con una operación de ejecución, de modo que el dispatch deje de ser un `match`
  manual sobre `args` y que un flag desconocido sea un error con código de salida distinto de 0, no
  un silencio (hoy `parse_run` descarta los tokens que no reconoce).
- **Validación al construir los argumentos de ejecución (C2)**: `--ticks` se valida como entero no
  negativo antes de construir los argumentos, eliminando la conversión a `u64` de un valor negativo
  que envuelve. `--every` conserva su validación actual (mayor o igual que 1).
- **Validación de `--root` (C3)**: si no se pasa `--root`, el valor por defecto se comprueba igual
  que uno explícito; si el directorio no existe, error claro en lugar de un fallo silencioso al
  cargar recursos.
- **Separación de flujos (C5)**: mapa y eventos de simulación por `stdout`; diagnósticos de carga,
  errores de argumentos y ayuda por `stderr`. Nada de simulación por `stderr` ni de error por
  `stdout`.
- **Pruebas (C4)** en `crates/instinct-cli/tests/`: `run` sin `--map`, `--root` inexistente,
  `--place` mal formado, especie desconocida, colocación fuera de límites, `--every <= 0`,
  `--ticks` negativo, flag desconocido y `check` con número incorrecto de argumentos.
- `cargo test -p instinct-cli`, clippy y fmt en verde.

**No hagas**

- No cambies el formato del mapa ni el de los eventos.
- No añadas subcomandos nuevos ni flags nuevos.
- No cambies los códigos de salida que ya fijan `cli_usage.rs` (2 para uso incorrecto) salvo para
  los casos nuevos que este issue define.
- No toques el núcleo salvo para adaptar la llamada de `draw` tras F2-07.

### Archivos previstos

- `crates/instinct-cli/src/main.rs` — modificar — comandos, validación, flujos.
- `crates/instinct-cli/tests/cli_run.rs` — modificar — batería C4 de `run`.
- `crates/instinct-cli/tests/cli_check.rs` — modificar — número incorrecto de argumentos.
- `crates/instinct-cli/tests/cli_usage.rs` — modificar — flag desconocido y ayuda.

## Criterios de aceptación

- [ ] **AC1** Un flag desconocido produce error y código de salida distinto de 0. Verif: `test_cli_unknown_flag_errors` | `cargo test -p instinct-cli --test cli_run` | prueba pasa.
- [ ] **AC2** `--ticks` negativo produce error y no se convierte a `u64`. Verif: `test_cli_negative_ticks_errors` | `cargo test -p instinct-cli --test cli_run` | prueba pasa.
- [ ] **AC3** `--root` inexistente produce error claro nombrando el directorio. Verif: `test_cli_missing_root_errors` | `cargo test -p instinct-cli --test cli_run` | prueba pasa.
- [ ] **AC4** `--every <= 0` sigue rechazándose como hasta ahora. Verif: `test_cli_every_zero_errors` | `cargo test -p instinct-cli --test cli_run` | prueba pasa.
- [ ] **AC5** Mapa y eventos salen por `stdout`; diagnósticos y errores por `stderr`. Verif: `test_cli_streams_separated` | `cargo test -p instinct-cli --test cli_run` | prueba pasa.
- [ ] **AC6** La batería C4 queda cubierta: `run` sin `--map`, `--place` mal formado, especie desconocida, colocación fuera de límites y `check` con número incorrecto de argumentos. Verif: `cargo test -p instinct-cli` | todas pasan, lista citada en las notas.
- [ ] **AC7** Los escenarios E2E de F0 y F1 siguen pasando con la misma salida por `stdout` para la misma semilla. Verif: `test_cli_determinism_same_seed` | `test_cli_f0_walker_no_regression` | `cargo test -p instinct-cli` | pasan.
- [ ] **AC8** Suite completa sin regresiones: clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `main` con dispatch manual sobre `args[1]`, `execute_run`, `execute_check`,
`parse_run`, `parse_place`, `RunArgs { root, map_name, seed, ticks, every, places }` y el uso
`const USAGE`; `--every` ya valida el mínimo; los tests `cli_usage.rs`, `cli_check.rs` y `cli_run.rs`
con los E2E de F1 y los de determinismo; en F2-03 `World::draw` pasó a devolver `Result<String,
Diagnostic>` y en F2-07 el dueño de los catálogos cambió, así que la impresión del mapa se adapta
antes de validar flujos.
**Contratos que debes respetar**: el núcleo no imprime; la CLI es el adaptador; códigos de salida
existentes; el mapa en ASCII fila por fila con salto final.
**Ojo**: mover mensajes de
`stdout` a `stderr` puede cambiar la salida que alguna prueba compara
byte a byte; ajusta esas pruebas y cítalo en las notas, asegurándote de que lo que se compara sigue
siendo la simulación.

### Leer primero

- `crates/instinct-cli/src/main.rs` entero.
- `crates/instinct-cli/tests/cli_run.rs`, `cli_check.rs`, `cli_usage.rs` — estilo y aserciones existentes.
- `crates/instinct-core/src/world.rs` — `draw` tras F2-03 y F2-07.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` sección 6 y `docs/PATRONES_DISENO_ARQUITECTURA.md` sección 7.

## Especificación

Cada subcomando es un tipo con una operación que recibe los argumentos que le siguen y devuelve un
código de salida. Los argumentos de ejecución se construyen con validación: flags desconocidos
rechazados, `--ticks` entero no negativo, `--every` entero mayor o igual que 1, `--root` directorio
existente, `--map` obligatorio, `--place` con forma `Especie@x,y`. La impresión reparte: simulación
a `stdout` y diagnóstico a `stderr`.

## Pasos sugeridos (opcional)

1. Adapta la llamada de `draw` a lo que dejaron F2-03 y F2-07.
2. Extrae los subcomandos a tipos con su ejecución.
3. Añade la validación al construir los argumentos, con pruebas primero para cada rechazo.
4. Separa los flujos y ajusta las pruebas que comparan salida.
5. `cargo test -p instinct-cli` y el gate completo.

## Tests

Nivel: integración, ejecutando el binario compilado por Cargo. Casos como comportamientos: cada
entrada inválida produce error por `stderr` y código distinto de 0; la misma semilla produce la
misma salida por `stdout`. Qué NO mockear: el binario real y los recursos de `examples/`.
Determinismo: semilla fija. Datos: `crates/instinct-cli/tests/fixtures` y `examples/`.

## Verificación

- `cargo test -p instinct-cli` → todas las pruebas de la CLI pasan.
- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si separar flujos rompe un E2E que compara salida combinada, conserva la comparación de la
simulación por `stdout` y ajusta la prueba. Decisiones que NO puedes tomar tú: añadir flags,
cambiar el formato del mapa o mover la impresión de eventos al núcleo.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Cierre documental de sesión única; el código está en `feat/f2-08-cli-validacion`, commit `fcf4c1e` (árbol limpio). Ejecutado desde la raíz del worktree con PowerShell (sin `&&`).

Puertas de calidad (todas exit 0):
- `cargo test --workspace` → todo en verde: 28 pruebas de CLI (6 cli_check + 19 cli_run + 3 cli_usage) y todo el core; EXIT_CODE=0.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings.
- `cargo fmt --all --check` → exit 0.

E2E runbook §12:
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` → EXIT_CODE=0; Walker recorre (0,0) a (10,0) tick a tick y muere por vida al final del tick 9 (`[tick 9] Walker@10,0 muere por vida`); stdout lleva mapa fila a fila y eventos; stderr vacío en corrida válida. Misma salida que F1 salvo la separación de flujos de F2-08.

Cero `unwrap`/`expect`/`panic!` en `crates/instinct-cli/src` (`grep -rn "unwrap()\|expect(\|panic!" crates/instinct-cli/src` sin coincidencias); los únicos `unwrap`/`expect` del crate están en `tests/` (harness del binario y aserciones de prueba), fuera de la restricción del runbook.

Evidencia AC por AC:
- AC1 `test_cli_unknown_flag_errors` (cli_run.rs): `run --bandera-inexistente` → código 2, stderr no vacío → PASS (`cargo test -p instinct-cli --test cli_run test_cli_unknown_flag_errors` ok).
- AC2 `test_cli_negative_ticks_errors` (cli_run.rs): `--ticks -3` → código 2, stderr nombra ticks; no se envuelve a u64 → PASS.
- AC3 `test_cli_missing_root_errors` (cli_run.rs): `--root no/existe/este/directorio` → código 2, stderr nombra el directorio → PASS.
- AC4 `test_cli_every_zero_errors` (cli_run.rs): `--every 0` → código 2 → PASS (validación previa mantenida).
- AC5 `test_cli_streams_separated` (cli_run.rs): `== tick 0 ==` en stdout; stderr vacío para corrida válida → PASS.
- AC6 batería C4 completa → PASS: `test_cli_missing_map_errors`, `test_cli_negative_ticks_errors`, `test_cli_missing_root_errors`, `test_cli_every_zero_errors`, `test_cli_malformed_place_errors`, `test_cli_unknown_species_errors`, `test_cli_place_out_of_bounds_errors` (cli_run.rs) y `test_cli_check_wrong_arg_count_errors` (cli_check.rs); `cargo test -p instinct-cli` → 28/28.
- AC7 `test_cli_determinism_same_seed` y `test_cli_f0_walker_no_regression` (cli_run.rs) → PASS; misma semilla, stdout byte a byte idéntico; sin regresiones de F0/F1.
- AC8 puertas anteriores → PASS (exit 0 en las tres).
- Códigos de salida de `cli_usage.rs` sin cambios: 2 para uso incorrecto, 1 para error de dominio, 0 para éxito; todas las pruebas lo verifican.

Pruebas ajustadas por cambio de flujos: `test_cli_prints_map`, `test_cli_streams_separated` y `test_cli_f0_walker_no_regression` asumen simulación/eventos en stdout y stderr vacío (C5); ninguna prueba compara salida combinada stdout+stderr.
