---
id: F5-01
title: Mapa con carácter desconocido rechazado, con carácter y línea
phase: F5
status: todo
depends_on: []
size: M
---

# F5-01 · Mapa con carácter desconocido rechazado, con carácter y línea

## Objetivo

Que un `.map` que use un carácter que ningún terreno ni objeto cargado declara quede rechazado por
completo, con un mensaje que nombra el carácter y la línea donde aparece, mientras los demás mapas
de la carpeta siguen cargando. Hoy la implementación solo descarta la fila y acepta el mapa, lo que
contradice el enunciado y el criterio de salida de F5.

## Alcance

**Haz**
- `load_map` rechaza el mapa completo cuando encuentra un carácter que no declara ningún `.te` ni `.ob` cargado: el diagnóstico nombra el carácter ofensor y la línea de la fila, y la función no devuelve mapa.
- Se reportan todos los caracteres desconocidos encontrados (uno por carácter y fila), no solo el primero, antes de rechazar el mapa.
- `load_maps_folder` sigue siendo fail-soft: un mapa rechazado no impide cargar los demás.
- La CLI refleja el rechazo: ese mapa no está disponible y sus diagnósticos salen por stderr.
- Los dos tests que afirman el comportamiento incorrecto se reescriben para afirmar el rechazo (autorización explícita más abajo en "Ojo").

**No hagas**
- No cambies el resto de validaciones del mapa (filas de longitud desigual, mapa sin filas): siguen como están.
- No cambies el formato de los mensajes existentes para otros errores de recursos.
- No toques `compile` ni los errores de ejecución: son otros issues.
- No añadas validación de nombres duplicados de mapa: decisión 5 de la fase.

### Archivos previstos
- `crates/instinct-core/src/resources.rs` — modificar — `load_map` rechaza el mapa y acumula los diagnósticos de carácter desconocido.
- `crates/instinct-core/tests/resources_tests.rs` — modificar — `test_map_unknown_char_reports_line` pasa a afirmar el rechazo.
- `crates/instinct-core/tests/world_tests.rs` — modificar — `test_unknown_character_generates_diagnostic` pasa a afirmar el rechazo.
- `crates/instinct-cli/tests/cli_run.rs` — modificar — caso nuevo de mapa rechazado por carácter desconocido vía `run`.

## Criterios de aceptación

- [ ] **AC1** Un `.map` con un carácter no declarado queda rechazado: `load_maps_folder` no lo incluye en los mapas cargados. Verif: `test_map_rejected_when_unknown_character` | `cargo test -p instinct-core --test resources_tests`.
- [ ] **AC2** El diagnóstico nombra el carácter ofensor y la línea correcta (línea 2 = primera fila de celdas, porque la línea 1 es el nombre del mapa). Verif: `test_map_rejection_reports_char_and_line` | `cargo test -p instinct-core --test resources_tests`.
- [ ] **AC3** Un mapa con dos caracteres desconocidos en filas distintas produce dos diagnósticos, uno por fila con su carácter. Verif: `test_map_rejection_reports_all_unknown_chars` | `cargo test -p instinct-core --test resources_tests`.
- [ ] **AC4** Los demás mapas de la carpeta cargan con normalidad y una carpeta con un mapa rechazado y uno válido devuelve solo el válido. Verif: `test_rejected_map_does_not_block_others` | `cargo test -p instinct-core --test resources_tests`.
- [ ] **AC5** En la CLI, `run` con un `--root` que contiene el mapa inválido no lo ofrece como disponible y muestra el diagnóstico por stderr con archivo y línea. Verif: `test_cli_run_rejects_map_with_unknown_char` | `cargo test -p instinct-cli --test cli_run`.
- [ ] **AC6** El mapa bueno de `examples/maps/plain.map` sigue cargando y su simulación no cambia. Verif: `cargo test --workspace`.
- [ ] **AC7** Suite completa, clippy y fmt en verde. Verif: `cargo test --workspace` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo fmt --all --check`.

## Contexto

**Ya existe**: la carga fail-soft por carpeta en `resources.rs` (un archivo inválido se descarta y los demás cargan); el diagnóstico de carácter desconocido ya existe pero no rechaza el mapa.

**Contratos que debes respetar**: fail-soft de carga (un error nunca aborta la carga de los demás); mensajes existentes sin cambios de formato; línea desde 1 (línea 1 = nombre del mapa).

**Ojo**: `test_map_unknown_char_reports_line` (resources_tests) y `test_unknown_character_generates_diagnostic` (world_tests) afirman hoy que la fila se descarta y el mapa carga. Este issue **autoriza expresamente** reescribirlos: la semántica anterior contradecía el enunciado ("el mapa se rechaza"). No es una trampa: es el cambio pedido. Ningún otro test existente debe modificarse.

### Leer primero
- `crates/instinct-core/src/resources.rs` — `load_map`, `load_maps_folder` y el estilo de los diagnósticos de recursos.
- `crates/instinct-core/tests/resources_tests.rs` — `test_map_unknown_char_reports_line` y los tests de carga de carpeta (patrón de fixtures en memoria).
- `crates/instinct-core/tests/world_tests.rs` — `test_unknown_character_generates_diagnostic` y cómo `World::from_map` consume mapas.
- `crates/instinct-cli/tests/cli_run.rs` — `run_cli` y los tests de argumentos inválidos (patrón de asertos sobre stderr y exit code).

## Especificación

- Entrada: contenido del mapa y la lista de caracteres válidos (los `char` de terrenos y objetos cargados).
- Recorrido: para cada fila de celdas, cada carácter que no esté en la lista de válidos produce un diagnóstico con el carácter y el número de línea; si hay al menos un carácter desconocido, el mapa se rechaza (no se devuelve).
- Salida: sin mapa, con los diagnósticos acumulados (todos, ordenados por línea).
- Filas con longitud desigual siguen rechazándose como hoy; ese camino no cambia.
- La CLI ya lista mapas disponibles cuando `--map` no existe; el mapa rechazado simplemente no aparece en esa lista.

## Tests

Nivel: unitario (núcleo, en memoria, sin sistema de archivos) y un caso de integración CLI sobre una fixture nueva mínima (un `.map` con carácter inválido dentro de `crates/instinct-cli/tests/fixtures/`). Escribe primero los tests que fallan (hoy el mapa con carácter desconocido carga). No mockees el cargador: usa contenido en memoria como los tests existentes de `resources_tests.rs`.

## Verificación

- `cargo test -p instinct-core --test resources_tests` → esperado: pasan los tres casos nuevos de rechazo.
- `cargo test -p instinct-core --test world_tests` → esperado: pasa el test actualizado.
- `cargo test -p instinct-cli --test cli_run` → esperado: pasa el caso nuevo de CLI.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → esperado: exit 0.
- `cargo fmt --all --check` → esperado: exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs ni código muerto.

## Si te bloqueas

Si rechazar el mapa completo rompiera algún escenario E2E existente más allá de los dos tests autorizados, para y repórtalo: significa que un ejemplo dependía del comportamiento incorrecto. Decisiones que no puedes tomar: cambiar el resto de mensajes de recursos, validar nombres duplicados de mapa, tocar la carga de `.te`/`.ob`.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: comandos ejecutados y resultado, AC por AC.

### Notas de implementación

<!-- Rama, commit, comandos ejecutados y resultado, AC por AC, desviaciones. -->
