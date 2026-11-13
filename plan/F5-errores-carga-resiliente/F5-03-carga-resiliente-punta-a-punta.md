---
id: F5-03
title: Carga resiliente de punta a punta sobre una carpeta mixta
phase: F5
status: todo
depends_on: [F5-01]
size: M
---

# F5-03 · Carga resiliente de punta a punta sobre una carpeta mixta

## Objetivo

Demostrar con un `run` real que una carpeta con archivos inválidos de cada tipo se carga informando
cada error con su archivo y línea, y que la simulación funciona con los recursos válidos: el mapa
bueno se dibuja, la especie buena se coloca y actúa. Es la prueba del enunciado: "sigue funcionando
con los demás".

## Alcance

**Haz**
- Fixture nueva `crates/instinct-cli/tests/fixtures/mixed_root/` con carpetas de terrenos, objetos, mapas y criaturas mezclando un archivo inválido y uno válido de cada tipo.
- Caso de integración CLI que ejecuta `run` sobre esa fixture y comprueba: exit 0, un diagnóstico por archivo inválido en stderr con ruta y línea, y mapa y simulación en stdout.
- Caso de especie inválida: no se puede colocar (mensaje de especie desconocida) mientras la especie válida sí funciona.
- Caso de colocación sobre objeto u otra criatura: se rechaza con diagnóstico antes de simular.
- Caso de carpeta de criaturas ausente: se trata como vacía, sin diagnóstico, y `run` funciona (contrato A2, no debe romperse).

**No hagas**
- No cambies la carga del núcleo (`resources.rs`): su fail-soft ya existe; este issue lo demuestra.
- No cambies la CLI ni sus mensajes.
- No crees criaturas inválidas nuevas más allá de una: reutiliza el formato de la batería de F5-02 si ya está; si no, una basta para este issue.
- No toques errores de ejecución: es F5-04.

### Archivos previstos
- `crates/instinct-cli/tests/fixtures/mixed_root/` — nuevo — carpetas con válidos más un inválido por tipo.
- `crates/instinct-cli/tests/cli_run.rs` — modificar — casos nuevos de carga mixta.

## Criterios de aceptación

- [ ] **AC1** Con la fixture mixta, cada archivo inválido produce exactamente un diagnóstico con su ruta y línea en stderr, y `run` termina con exit 0. Verif: `test_cli_run_mixed_root_reports_and_survives` | `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC2** El mapa válido de la fixture se dibuja con el encabezado de tick 0 y la criatura válida se coloca y emite al menos un evento durante los ticks configurados. Verif: `test_cli_run_mixed_root_reports_and_survives` | `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC3** Colocar la especie del archivo inválido falla con el mensaje de especie desconocida (exit 1); la especie válida se coloca sin error. Verif: `test_cli_run_invalid_species_not_placeable` | `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC4** Una colocación sobre una casilla con objeto o con otra criatura se rechaza con diagnóstico antes de simular (exit 1). Verif: `test_cli_place_on_obstacle_errors` | `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC5** Un root sin carpeta de criaturas corre sin diagnósticos de carga y sin criaturas colocables. Verif: `test_cli_run_missing_creatures_folder_is_empty` | `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC6** El root `examples` sigue dando la misma salida determinista con semilla fija. Verif: `cargo test -p instinct-cli --test cli_annex`.
- [ ] **AC7** Suite completa, clippy y fmt en verde. Verif: `cargo test --workspace` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo fmt --all --check`.

## Contexto

**Ya existe**: `resources::load_resources` fail-soft por archivo y carpeta; la CLI imprime diagnósticos de carga a stderr y eventos a stdout; los casos de colocación inválida, especie desconocida y fuera de límites en `cli_run.rs` (patrón a imitar). F5-01 garantiza que el mapa inválido de la fixture queda rechazado; este issue lo integra con el resto.

**Contratos que debes respetar**: fail-soft de carga; carpeta ausente = vacía (A2); separación de streams (C5: diagnósticos a stderr, simulación a stdout); un diagnóstico por archivo inválido con ruta y línea.

**Ojo**: la fixture debe ser determinista y no depender de `random`; usa como modelo la criatura caminante determinista de `examples/creatures/walker.ins`. No la enlaces: cópiala, para que la fixture no dependa de `examples/`.

### Leer primero
- `crates/instinct-cli/src/main.rs` — `execute_run`: dónde se imprimen diagnósticos y cómo se buscan mapa y especies.
- `crates/instinct-cli/tests/cli_run.rs` — `run_cli`, `test_cli_events_stdout_diagnostics_stderr` y `test_cli_streams_separated` (asertos sobre stderr y stdout).
- `crates/instinct-core/tests/resources_tests.rs` — formato de los recursos válidos mínimos (terreno, objeto, mapa y criatura) reutilizado por las fixtures.
- `examples/creatures/walker.ins` — criatura simple y determinista para la fixture.

## Especificación

Composición mínima de `mixed_root/` (un inválido más un válido por tipo):
- Terrenos: uno válido y uno con clave obligatoria ausente.
- Objetos: uno válido y uno con valor no entero en `resource_max`.
- Mapas: uno válido y uno con carácter desconocido, rechazado por F5-01.
- Criaturas: una válida (determinista) y una con error de compilación de la batería.

Comportamiento esperado de `run` sobre la fixture con el mapa bueno, la criatura buena colocada en una casilla libre, tres ticks y semilla fija:
- stderr: un diagnóstico por archivo inválido, con su ruta y línea; sin más ruido.
- stdout: el encabezado de tick 0, el mapa dibujado y los eventos de los ticks.
- exit 0.

## Tests

Nivel: integración CLI contra el binario real con la fixture en disco. Escribe primero los tests que fallan hoy (la fixture aún no existe). No mockees el cargador ni la CLI; los diagnósticos deben salir del camino real de carga.

## Verificación

- `cargo test -p instinct-cli --test cli_run` → esperado: pasan los cuatro casos nuevos.
- `cargo test -p instinct-cli --test cli_annex` → esperado: sin regresiones en los ejemplos.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → esperado: exit 0.
- `cargo fmt --all --check` → esperado: exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde.

## Si te bloqueas

Si la fixture mixta hace que `run` falle por una razón distinta a los diagnósticos esperados, repórtalo: puede ser un hueco real de la carga fail-soft. Decisiones que no puedes tomar: cambiar la política de carpetas ausentes, alterar la separación de streams, tocar `examples/`.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: comandos ejecutados y resultado, AC por AC.

### Notas de implementación

<!-- Rama, commit, comandos ejecutados y resultado, AC por AC, desviaciones. -->
