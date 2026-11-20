---
phase: F5-ventana
title: Crear y conectar la parte visual (ventana)
status: planned
master: ../MASTER.md
fuentes:
  - ./INFORME_01_API_NUCLEO.md
  - ./INFORME_02_FLUJO_GUI.md
  - ./INFORME_03_DESACOPLAMIENTO.md
  - ./INFORME_04_PROPUESTA_ISSUES.md
  - ./INFORME_05_DECISIONES_ABIERTAS.md
  - ../../docs/enunciado.md
  - ../../docs/PATRONES_DISENO_ARQUITECTURA.md
issues: 8 desglosados en archivos propios (F5-01 a F5-08)
---

# F5 · Ventana (crear y conectar la parte visual)

## Objetivo y entregable

Construir el tercer crate del workspace, `instinct-gui`, y conectarlo al núcleo por los
puertos ya existentes (snapshot de solo-lectura, bus de eventos `WorldOutput`,
`Simulation::step_tick`) **sin cambiar el núcleo salvo lo que un issue declare**.
La ventana cumple el flujo de la sección 4.1 del enunciado: carga de recursos con
diagnósticos con línea sin abortar, preparación (elegir mapa, colocar/quitar criaturas),
simulación con play/pausa/paso/3 velocidades/reinicio, panel de registro y visualización
de los rugidos (sección 2.4).

Entregable observable: `cargo run -p instinct-gui` abre una ventana que demuestra el
flujo completo de 4.1; `cargo test --workspace` cubre la lógica de la ventana sin
abrir gráfico (ver "Principio rector" más abajo).

## Principio rector (desacoplamiento y buenos principios)

1. **El núcleo no cambia ni se enterará de que existe la ventana.** `instinct-gui`
   depende de `instinct-core` (nunca al revés). Cero cambios de API salvo declaración
   expresa en un issue, aprobada con test que falle primero.
2. **La GUI es un adaptador, no un dueño.** Los eventos entran por el puerto
   `WorldOutput` (`event/output.rs`); el estado del mundo se lee por el DTO
   `WorldSnapshot` (`world/snapshot_view.rs`). La GUI nunca muta el mundo salvo por
   su propia instancia (`place_creature` / `remove_creature` / `step_tick`).
3. **La lógica de la ventana es headless y testeable.** Toda regla de estado (fases de
   la app, colocaciones, control de tiempo, reinicio) vive en módulos puros de
   `instinct-gui` sin conocimiento de `eframe`; la capa `eframe` solo traduce estados a
   widgets y clics a mensajes. Los tests de integración corren la máquina de estados
   completa sin abrir ventana.
4. **Determinismo intacto.** Toda semilla producida por la GUI debe dar la misma
   historia que la CLI con esa semilla. El reinicio recrea el mundo desde `from_map`
   + `place_creature` + `set_rng_seed(seed)` (nunca `World::clone`, que resetea el RNG
   a 0: `world/clone_debug.rs:24`).
5. **Puertas verdes tras cada issue**; `git status` limpio tras commit.

## Estado de partida (verificado por el orquestador, 2026-11-19)

- `develop` limpio en 81f3cf1. `cargo test --workspace`, `cargo clippy --workspace
  --all-targets -- -D warnings` y `cargo fmt --all --check` en verde.
- Línea base de determinismo (`.opencode/ledgers/f5-ventana-baseline.txt`):
  `instinct-cli run --root examples --map 'Plain with rocks' --seed 7 --ticks 30
  --place Dwarf@1,0 --place Ent@5,1 --place Hobbit@10,0 --place Uruk@12,3` →
  SHA256 `BF6DE0F5EAB874DF037CE3B8AC9B996B4115F55FAF6D77379531E88F00656D09` (2 corridas
  idénticas, 284 líneas, stderr vacío).
- El núcleo ya expone: `WorldSnapshot` + `World::snapshot()` (R2-02), `WorldOutput`
  (F4-01), `Simulation::step_tick/set_rng_seed`, `place_creature`/`remove_creature`
  (valida bounds y celda ocupada: `world/life.rs:20-85`), `previous_roars()`
  (`world/roars.rs:38`), `load_resources` (`resources/mod.rs:47`).
- Huecos confirmados (no existen hoy): ventana, estado de app, velocidad, reinicio.
- Red disponible al planificar: las dependencias gráficas se descargan en F5-01
  (atajo 4 de MASTER: descargarlas en la primera conexión antes de empezar).

## Decisiones de la fase

1. **Repartición de la fase:** MASTER preveía F5 = Ventana + Banco (9–13 issues). El
   banco de criaturas (issue F5-11 de INFORME_04) **sale de esta fase** y se planificará
   como su propia fase (`F6-banco`), por la regla de replanificación de MASTER ("una
   fase con más de 8 issues: dividirla"). Esta fase se acota a la ventana: 8 issues.
2. **Biblioteca gráfica: `egui` + `eframe`**, validada en F5-01 compilando sin red una
   vez descargadas (pura Rust, sin bibliotecas de sistema en Windows; DECISION_ABIERTA_1
   resuelta por este plan).
3. **Crate GUI:** `crates/instinct-gui` (DECISION_ABIERTA_2), tercer miembro del
   workspace; sus propias `[[test]]` por área, patrón `tests/<area>/mod.rs`.
4. **Reinicio:** recrear mundo desde `from_map` + colocaciones + `set_rng_seed(seed)`
   (DECISION_ABIERTA_3). La GUI guarda la "receta" (mapa, colocaciones, semilla), no el
   mundo.
5. **Velocidades: 2 / 10 / 30 ticks por segundo** exactamente (DECISION_ABIERTA_4,
   ejemplo del enunciado 4.1).
6. **El núcleo mantiene 0 dependencias y 0 E/S**; las dependencias gráficas viven solo
   en `instinct-gui` (DECISION_ABIERTA_5).
7. **Verificación headless del flujo:** `instinct-gui` expone un modo `--smoke` que
   ejecuta el flujo 4.1 completo sobre la máquina de estados sin abrir ventana y
   termina con exit 0. Es la puerta automatizable del entregable observable; la
   demostración visual con ventana real es manual (criterio del issue F5-08).
8. **Fusión de issues propuestos** para respetar el techo de 8: spike+crate (F5-01) y
   adaptador de eventos+panel de registro (F5-02) van juntos porque son inseparables en
   la práctica (un puerto sin panel no es observable; un panel sin puerto no existe).

## Issues

| ID | Título | Problema / patrón que cubre | Depende de | Tamaño | Estado | Archivo de issue |
|----|--------|----------------------------|------------|--------|--------|------------------|
| F5-01 | Spike egui/eframe offline + crate `instinct-gui` en el workspace | Elegir biblioteca (4.1 depende de ella), compilar sin red con deps descargadas; rejilla + clic; alta del crate | — | M | done | f5-01-spike-gui-crate.md |
| F5-02 | Adaptador `WorldOutput` para GUI + panel de registro | Conectar el puerto de eventos a un buffer observable; registro con carga, ejecución y eventos | F5-01 | M | done | f5-02-adaptador-eventos-registro.md |
| F5-03 | Pantalla de carga y catálogos | Cargar recursos y listar mapas/especies con diagnósticos y línea sin abortar (4.1 paso 1) | F5-01, F5-02 | M | done | f5-03-carga-catalogos.md |
| F5-04 | Preparación: elegir mapa, colocar y quitar criaturas | 4.1 paso 2: colocaciones válidas y removibles sobre la máquina de estados | F5-03 | M | done | f5-04-preparacion-colocaciones.md |
| F5-05 | Controles de tiempo: play, pausa, paso y velocidades | 4.1 paso 3: máquina de estados del tiempo; 2/10/30 ticks por segundo | F5-04 | M | done | f5-05-controles-tiempo.md |
| F5-06 | Render del snapshot en la ventana | DTO `WorldSnapshot` → rejilla, terrenos, objetos, criaturas | F5-01, F5-02 | M | done | f5-06-render-snapshot.md |
| F5-07 | Visualización de rugidos | 2.4: "la interfaz visual debe mostrarlo de alguna manera" | F5-06 | S | done | f5-07-rugidos-visibles.md |
| F5-08 | Reinicio a preparación + verificación de flujo headless | 4.1 paso 4: receta (mapa, colocaciones, semilla) + `--smoke` del flujo completo | F5-04, F5-05 | M | done | f5-08-reinicio-verificacion-flujo.md |

### Orden y dependencias

- F5-01 es raíz: sin ella no hay crate GUI ni biblioteca.
- F5-02 depende de F5-01 (el puerto y el panel viven en el crate nuevo).
- F5-03 depende de F5-01 y F5-02 (la carga alimenta el registro).
- F5-04 depende de F5-03 (la preparación opera sobre recursos cargados).
- F5-05 depende de F5-04 (el tiempo solo corre con criaturas colocadas).
- F5-06 depende de F5-01 y F5-02 (render puro sobre el DTO; independiente de la carga).
- F5-07 depende de F5-06 (destacar sobre la rejilla renderizada).
- F5-08 cierra: reinicio + verificación headless de los cuatro pasos de 4.1.

## Cobertura

| Elemento | Dónde se cubre |
|----------|----------------|
| Biblioteca gráfica offline + crate GUI | F5-01 |
| Puerto de eventos + registro headless | F5-02 |
| Carga de recursos y diagnósticos (4.1 paso 1) | F5-03 |
| Preparación y colocaciones (4.1 paso 2) | F5-04 |
| Play/pausa/paso/velocidades (4.1 paso 3) | F5-05 |
| Render del mundo (DTO de solo-lectura) | F5-06 |
| Rugidos visibles (2.4) | F5-07 |
| Reinicio + flujo 4.1 headless (paso 4) | F5-08 |

## Verificación de salida de la fase

- `cargo test --workspace` → exit 0 (incluye los tests de `instinct-gui`).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- `cargo run -p instinct-gui -- --smoke` → recorre carga → preparación → play/pausa/paso/
  velocidades → reinicio → registro y sale con exit 0.
- Determinismo: la GUI con semilla S produce la misma historia que la CLI con semilla S
  (2 corridas `--seed 7` idénticas al baseline `BF6DE0F5…6D09`; el núcleo no cambia).
- `crates/instinct-core`: sigue con 0 dependencias externas y 0 `println!`.
- `git status --short` limpio tras cada commit.
- Demostración manual con ventana real (guiía en `plan/F5-ventana/DEMO_MANUAL.md`,
  completada en F5-08): el flujo 4.1 se ve y se opera.

## Prompt de sesión

```text
Implementa el issue plan/F5-ventana/<ID>-<slug>.md.
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
La ventana se construye desacoplada: la lógica headless se prueba sin abrir gráfico y
la capa eframe solo traduce estados a widgets. No cambies el núcleo salvo que el issue
lo declare con test que falle primero. Ejecuta los comandos de "Verificación" y no lo
des por terminado hasta cubrir cada criterio de aceptación con evidencia. Al acabar:
actualiza `status` y rellena "Notas de implementación". Si el issue contradice el repo
o te bloquea, para y repórtalo en lugar de improvisar.
```

## Registro de ejecución

| Issue | Patrón / enfoque | Motivación | Coste asumido | Estado | Medición |
|-------|------------------|------------|---------------|--------|----------|
| F5-01 | egui+eframe 0.33.3 + crate `instinct-gui`; geometría render pura y clic en panel mínimo | Elegir biblioteca gráfica, compilar sin red, alta del crate GUI en el workspace | M (spike) | done | `cargo build --offline -p instinct-gui` → exit 0; `cargo test --workspace` → exit 0; clippy/fmt → exit 0 |
| F5-02 | `LogBuffer` adaptador `WorldOutput` headless (sin eframe/egui) + modelo de panel de registro; tests de integración por área y modo `--smoke` en `main.rs` | Conectar el puerto de eventos al GUI sin tocar el núcleo; registro acotado y testeable sin ventana | M (adaptador + tests) | done | `cargo test -p instinct-gui --test log` → 8/8 ok; `cargo test --workspace` → exit 0; clippy/fmt → exit 0; `cargo run -p instinct-gui -- --smoke` → exit 0 con `[tick 0] Talker@1,0 says: hola` vía puerto |
| F5-03 | `LoadedSession` headless sobre `load_resources` + catálogos (mapas/especies) ordenados alfabéticamente, selección por índice y registro de diagnósticos de carga; panel lateral en eframe; `--root`/`--smoke` en `main.rs` | Carga de recursos con diagnósticos con línea sin abortar; app no entra en preparación sin mapas ni especies | M (estado + tests + UI panel) | done | `cargo test -p instinct-gui --test session` → 6/6 ok (T1-T5); `cargo test --workspace` → exit 0; clippy/fmt → exit 0; `cargo run -p instinct-gui -- --smoke` → exit 0 |
| F5-04 | Estado de preparación headless (PrepState + Recipe) sobre máquina de estados existente; UI solo mensajes | 4.1 paso 2: colocaciones válidas y removibles sin tocar el núcleo | M (estado + tests + UI panel) | done | `cargo test -p instinct-gui --test prep` → 11/11; `cargo test --workspace` → exit 0; clippy/fmt → exit 0; git diff instinct-core vacío |
| F5-05 | `TimeControl` headless (`Speed` 2/10/30 tps, `RunState` Idle/Running/Paused, `advance(elapsed_ms) -> n_ticks`) + integración en `AppState` con `sim` y `log_buffer`; UI con play/pausa/paso/velocidades/tick | 4.1 paso 3: máquina de estados del tiempo; 2/10/30 ticks por segundo | M (estado + tests + UI panel) | done | `cargo test -p instinct-gui --test time` → 18/18 ok; `cargo test --workspace` → exit 0; clippy/fmt → exit 0; git diff instinct-core vacío |
| F5-06 | `render/` puro sobre `WorldSnapshot`: `grid_from_snapshot` + `CellView`/`GridView`/`Rgb`/`CellKind` (prioridad criatura > objeto > terreno, símbolos del DTO, metadatos ticks/map_name); `ui/` pinta la rejilla cada frame desde `snapshot()` (sim o preparación) con `rect_filled` + símbolo centrado, sin lógica de simulación | DTO headless + tests + capa eframe solo traduce estado→widgets | M (render + UI) | done | `cargo test -p instinct-gui --test render` → 10/10; `cargo test --workspace` → exit 0; clippy/fmt → exit 0; `git diff develop -- crates/instinct-core/` vacío |
| F5-07 | `roar_markers` puro en `render/roars.rs` + overlay ámbar en `ui/mod.rs` + `last_events` en `AppState` para eventos de `step_tick` | Rugidos visibles sobre la rejilla sin tocar el núcleo; tests headless con `examples/` y Roarer | S (overlay + tests) | done | `cargo test -p instinct-gui --test render` → 17/17; `cargo test --workspace` → exit 0; clippy/fmt → exit 0; `git diff develop -- crates/instinct-core/` vacío |
| F5-08 | `AppState::restart()` sobre la máquina de estados (recrea el mundo desde `recipe` = mapa + colocaciones + semilla, nunca `World::clone`); `run_smoke` del flujo 4.1 completo + puerta de determinismo (GUI == núcleo con `--seed 7`); botón "Reiniciar" en `ui/` + `DEMO_MANUAL.md` | 4.1 paso 4 (el botón devuelve a preparación) y puerta automatizable del entregable observable; determinismo intacto sin tocar el núcleo | M (restart + smoke + tests + UI + demo) | done | `cargo test -p instinct-gui --test reinicio` → 3/3, `--test determinismo` → 1/1; `cargo test --workspace` → exit 0; clippy/fmt → exit 0; `cargo run -p instinct-gui -- --smoke` → exit 0 terminando en `== --smoke OK: flujo 4.1 completo verificado ==`; 2 corridas CLI `--seed 7` byte a byte idénticas (SHA256 captura CRLF `BF6DE0F5…6D09`, stderr vacío); `git diff develop -- crates/instinct-core/` vacío |

<!-- Se rellena al avanzar: enfoque, motivación, coste, mediciones. -->