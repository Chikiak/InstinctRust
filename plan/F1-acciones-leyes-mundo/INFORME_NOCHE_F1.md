# Informe nocturno — Fase F1 · Acciones y leyes del mundo

Sesión nocturna ejecutada contra `RUNBOOK_F1.md` sobre rama base `develop`.
Inicio 01:52 (Wed 04 Nov 2026) · cierre ~03:4x · resultado: **F1 completa, 5/5 issues done**.

## 1. Estado por issue

| Issue | Título | Estado | Rama | Merge a develop |
|---|---|---|---|---|
| F1-01 | `attack`: daño a criatura, objeto, terreno y casilla propia | **done** | feat/f1-01-attack-dano-criatura-objeto-terreno (borrada) | 20844d9 |
| F1-02 | `consume`: igual que attack más curación del atacante | **done** | feat/f1-02-consume-dano-mas-curacion (borrada) | 2a41658 |
| F1-03 | `reproduce`: cría, condiciones y aborto natural | **done** | feat/f1-03-reproduce-cria-condiciones-aborto (borrada) | a9253ad |
| F1-04 | `roar`: mensaje disponible el tick siguiente | **done** | feat/f1-04-roar-mensaje-tick-siguiente (borrada) | fe7f5c1 |
| F1-05 | Escenarios E2E uno por acción desde la terminal | **done** | feat/f1-05-escenarios-e2e-terminal (borrada) | d5a3d99 |

Ningún issue quedó `blocked`. Cadena íntegra, sin bloqueos ni rescates: los 5 salieron en
verde en el **intento 1** del implementador, cada uno con un solo ciclo
implementar → verificar → merge. Commits por issue (código y docs de estado
separados en F1-02, F1-03, F1-04 y F1-05):

- F1-01: `4907514` feat → merge `20844d9`
- F1-02: `1ca59aa` feat + `38df194` docs → merge `2a41658`
- F1-03: `5832a41` feat + `c36b2e9` docs → merge `a9253ad`
- F1-04: `1d79777` feat + `b48dd7d` docs → merge `fe7f5c1`
- F1-05: `797a297` feat + `a901aa6` docs → merge `d5a3d99`

Un commit del orquestador: `9c4a746` (pre-vuelo) y `b411c99` (cierre del ledger).

## 2. AC verdes por issue (copiados de las Notas de implementación)

- **F1-01 (9/9)**: AC1 test_attack_kills_creature_immediately · AC2 test_attack_wounds_creature ·
  AC3 test_attack_damages_object · AC4 test_attack_damages_terrain ·
  AC5 test_attack_out_of_map_noop · AC6 test_attack_bad_direction_errors ·
  AC7 test_attack_text_argument_errors · AC8 test_catalog_resolves_attack ·
  AC9 gate completo (`cargo test --workspace`, clippy exit 0, fmt exit 0).
  Evidencia: `cargo test -p instinct-core --test actions_tests`.
- **F1-02 (9/9)**: AC1 consume_damages_creature · AC2 consume_heals_attacker ·
  AC3 consume_damages_object · AC4 consume_damages_terrain · AC5 consume_out_of_map_noop ·
  AC6 consume_bad_direction_errors · AC7 consume_text_argument_errors ·
  AC8 catalog_resolves_consume · AC9 gate completo. Curación sin tope de vida, aplicada tras el daño.
- **F1-03 (9/9)**: AC1 reproduce_creates_offspring · AC2 reproduce_mother_loses_health ·
  AC3 reproduce_offspring_no_variables · AC4 reproduce_offspring_starts_at_start_next_tick (sim_tests) ·
  AC5 abort por casilla ocupada / por objeto · AC6 abort fuera del mapa ·
  AC7 abort por salud insuficiente · AC8 catalog_resolves_reproduce · AC9 gate completo.
  Cría con `sleep_turns = 0`, `pc = 0`, variables vacías, primer turno en el tick siguiente.
- **F1-04 (6/6)**: AC1 roar_emits_event (`Roared` con tick, especie, posición y texto) ·
  AC2 `roar(42)` → "42" · AC3 consumes_turn == true ·
  AC4 roar_stored_with_tick + test_roar_rotates_to_previous_tick (tick N legible en N+1) ·
  AC5 catalog_resolves_roar (aridad 1) · AC6 gate completo.
- **F1-05 (7/7)**: AC1 cli_e2e_attack_kills_adjacent_creature · AC2 cli_e2e_consume_kills_and_heals ·
  AC3 cli_e2e_reproduce_birth · AC4 cli_e2e_roar_emits_event ·
  AC5 dos corridas `--seed 1 --ticks 10` byte a byte idénticas (SHA-256) ·
  AC6 cli_f0_walker_no_regression · AC7 gate completo.

## 3. Desviaciones aplicadas y justificación

1. **Pre-vuelo con árbol sucio (corregido aquí).** `git status` mostraba cambios
   documentales de F0 sin commitear (`status: todo → done` en 3 ficheros) y
   `plan/F1-acciones-leyes-mundo/` entero sin trackear. Se comiteó como
   `chore: cierre documental de F0 y carga del plan de F1` (9c4a746) para partir
   de árbol limpio, como pide el runbook. No toca código ni contratos.
2. **`cargo test --workspace --nologo` no existe en este toolchain** (error
   `unexpected argument '--nologo'`). Se ejecuta `cargo test --workspace`, que es
   lo mismo sin el flag. Desviación de comando, no funcional.
3. **Auditoría del reviewer (§7.5) sin agente `reviewer`.** Este entorno solo
   expone `implementador-nemotron`, `rescatista-step5` y `verificador-nemotron`.
   Un primer intento la despachó por error con el rescatista (perfil incorrecto:
   su misión es terminar issues fallados) y dos despachos murieron por
   **infraestructura** (reinicio del servidor), nunca por trabajo del modelo. La
   auditoría se hizo finalmente con `verificador-nemotron`, que tiene el perfil
   exacto que pide el runbook: solo lectura + bash, sin `edit`. No se modificó
   nada en esas sesiones.
4. **Corrección no planificada dentro de F1-05.** Al integrar el escenario E2E de
   `attack` se detectó un bug real de la F1-01: `Died` por golpe reportaba la
   especie del **atacante** en lugar de la víctima, inconsistente con
   `end_of_tick_creatures`/`die_by_error`. Corregido en `strike_cell`
   (`actions.rs`), 5 líneas, dentro del alcance del issue que lo descubrió. No
   cambia contratos.

## 4. Verificación independiente

Sesión fresca de `verificador-nemotron` sobre los 5 issues done: **5/5 PASS**.
`status: done` en frontmatter y tabla de PHASE.md; notas rellenas con evidencia
real (comando + resultado por AC, no genéricas); commits existentes. Gate
ejecutado por el verificador: `cargo test --workspace` exit 0,
`cargo clippy --workspace --all-targets -- -D warnings` exit 0,
`cargo fmt --all --check` exit 0. Sin `unwrap`/`expect`/`panic!` sobre datos de
usuario en el código nuevo de F1. Resultado asesor: la revisión matutina manda.

Nota de higiene: los dos subagentes contaron distinto el número de tests
(162 y 129) pero ambos con exit 0 y 0 fallos. Recuento del orquestador:
**150 tests, 0 fallos**.

## 5. Auditoría del reviewer (diff completo 9c4a746..HEAD)

**Veredicto: [APROBADO]**, sin hallazgos bloqueantes:

1. Pánicos: cero `unwrap`/`expect`/`panic!` sobre datos de usuario o fichero en
   `instinct-core/src`; los dos `panic!` son arms internos de test.
2. Aritmética: `checked_sub`/`checked_add` en toda la ruta de daño y curación; sin
   división/resto por cero ni desbordamiento.
3. `Catalog` itera un `Vec<Rc<dyn Action>>`; los `HashMap` son solo de lookup por
   etiqueta/variable, sin orden de salida dependiente.
4. Cero cadenas de `if` por nombre de acción; `attack`, `consume`, `reproduce` y
   `roar` registradas por nombre en `Catalog::default()`.
5. Cero `println!`/`eprintln!` en `instinct-core`; `write!` es solo `Display` de `Event`.
6. Sin `#[ignore]`, sin `#![allow]` que tape warnings, sin asserts vacíos ni comentados.
7. Sin dependencias externas nuevas en `instinct-core`; `rand` ausente; sin `f32`/`f64`.
8. Contratos compartidos cumplidos: muerte inmediata con `Died` causa `"vida"` en
   el mismo tick; fuera del mapa no-op que consume turno; (0,0) = terreno propio;
   cría edad 0, sin variables, `pc` 0, primer turno en el tick siguiente; `roar`
   disponible el tick siguiente; toda acción salvo `say` consume turno aunque falle.
9. Determinismo: `test_same_seed_same_history` y `test_cli_determinism_same_seed` pasan.

## 6. Verificación de salida de la fase (ejecutada por el orquestador)

- `cargo test --workspace` → **150 tests, 0 fallos**, exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, sin warnings.
- `cargo fmt --all --check` → exit 0.
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` → exit 0, sin regresión de F0.
- [x] Cada acción de F1 (`attack`, `consume`, `reproduce`, `roar`) tiene prueba unitaria con resultado observable, más escenario E2E desde la terminal.
- [x] Dos corridas con la misma semilla dan la misma historia (determinismo).
- [x] Sin pánicos con objetivos fuera del mapa, casilla propia u objeto agotado (auditado).
- [x] `docs/ambiguedades.md` (15 líneas de tabla): **ninguna fila sin decisión**; F1 no añadió ambigüedades nuevas.

## 7. Siguiente paso recomendado

F1 está cerrada y `develop` queda lista para F2 (`roars_near`/`last_roar ya tienen
su almacén y rotación en `world.rs`/`sim.rs`). Recomendado:

1. Revisión matutina de los 5 merges en `develop` y de la corrección de
   `strike_cell` (§3.4), que es lo único que altera comportamiento previo.
2. Abrir F2 con el procedimiento estándar (`PHASE.md` §Prompt de sesión +
   `RUNBOOK_F2.md` cuando exista), aprovechando que el runbook de F1 no fija rama
   base para F2.
3. Nada que deshacer ni revertir: no hubo issues `blocked`, ni ramas descartadas,
   ni trabajo perdido.
