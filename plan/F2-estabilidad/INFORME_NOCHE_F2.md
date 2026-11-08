# Informe de la noche — Fase F2 (Estabilidad: bugs, contratos y pruebas)

Sesión nocturna del 05/11/2026 (cierre el 06/11). Orquestador + subagentes
(`implementador-nemotron`, `rescatista-step5` disponible sin uso, `verificador-nemotron`).
Rama: `develop`. Estado final: **fase completa, 8/8 issues done y fusionados**.
Desarrollo: `38930a2` (docs de la fase) → `7c9c207` (merge de F2-08).

## 1. Estado por issue

| ID | Estado | AC | Commit | Merge a develop |
|----|--------|----|--------|-----------------|
| F2-01 lexer: líneas, escapes, ASCII | done | 7/7 | `3a69837` | `a1b4377` |
| F2-02 parser: args, asignación, headers | done | 7/7 | `37a2c9a` | `c9129ba` |
| F2-03 errores unificados, cero unwrap | done | 7/7 | `4a7ac5e` | `a4d354d` |
| F2-04 etiquetas deterministas + núcleo | done | 6/6 | `325d51b` | `48c7543` |
| F2-05 intérprete: pc y límite de líneas | done | 6/6 | `e8b8a41` | `49a9804` |
| F2-06 mundo/tick: aritmética comprobada | done | 6/6 | `00cdf67` | `c2385a8` |
| F2-07 catálogos con un solo dueño | done | 7/7 | `17decbe` | `81897ea` |
| F2-08 CLI: validación y flujos | done | 8/8 | `fcf4c1e` + `1520f97` | `7c9c207` |

Ningún issue quedó `blocked`. No se usó `rescatista-step5`.

Cobertura de `docs/PROBLEMAS_Y_SOLUCIONES.md`: L1–L6, P1–P7, I1, I2, I4, I5,
W1–W4, W6, A1–A5 y C1–C5 resueltos en esta fase. Los problemas asignados a F4/F5
(I3, I6, W5) quedan fuera de alcance por diseño del plan.

Orden ejecutado (grafo del runbook §3, con paralelismo autorizado): F2-02 → luego
en paralelo **F2-01 + F2-06** → **F2-03 + (F2-06 aún en curso)** → en paralelo
**F2-04 + F2-05** → F2-07 → F2-08. Las tres raíces se cruzaron según disponibilidad
sin romper dependencias: F2-03 esperó a F2-01 (ambos tocan `lexer.rs`/`LexError`).

## 2. Cambios de firma (los cinco autorizados por PHASE.md) y su propagación

| Firma | Antes | Ahora | Propagación |
|-------|-------|-------|-------------|
| `lex` | `lex(text, line)` | `lex(text, line, file)` | Único sitio: `expr.rs:498` |
| `LexError` | `{ message, line }` | `{ file, message, line }` + `From<LexError> for Diagnostic` | `lexer.rs:52,68`, usado por `compile.rs` |
| `World::draw` | `draw(&self, terrains, objects) -> String` con `expect` | `draw(&self) -> Result<String, Diagnostic>` | CLI `main.rs:209,234`; tests con `.unwrap()` (harness) |
| `end_of_tick_terrain_and_objects` | recibía catálogos | `(&mut self) -> Vec<Pos>` | `sim.rs:68` |
| `Simulation::new` | `new(world, terrains, objects)` con copias propias | `new(world: World)`; `Simulation` tiene solo `pub world: World` | `sim.rs:20,26`; CLI y tests |

Además, firma interna decidida **dentro** de F2-02 y autorizada por el runbook §11:
`parse_action_call` retorna `ActionOutcome` (no `Option<Line>`) y `validate_header_value`
recibe el entero ya parseado. Y `Program::labels`: `HashMap` → `BTreeMap` (A4, F2-04).

## 3. Pruebas existentes ajustadas (revisadas, no regresiones silenciosas)

- `interp_tests.rs` (4 sitios): `HashMap::from` → `std::collections::BTreeMap::from`,
  exigido por `Program.labels` como `BTreeMap` (F2-04).
- `sim_tests.rs`, `world_tests.rs`, `cli_run.rs`: ajustadas al dueño único de
  catálogos y a las firmas de `draw`/`Simulation::new` (F2-07).
- `lexer_tests.rs`: ajustadas al parámetro `file` de `lex` y a `LexError` (F2-03).
- El verificador independiente confirmó que cada ajuste cambia **solo** lo que el
  cambio de contrato exigía y que ninguna aserción se suavizó.

## 4. Verificación independiente (verificador, sesión fresca)

**Veredicto global: [APROBADO]**

- 8/8 issues con todos sus AC en PASS contra el repo; 8/8 `status: done` en el
  archivo y en la fila de `PHASE.md`; 8/8 con "Notas de implementación" rellenas AC por AC.
- Puertas de la fase sobre `develop`: `cargo test --workspace` exit 0,
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0,
  `cargo fmt --all --check` exit 0,
  `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` exit 0.
- **Cero** `unwrap()`/`expect()`/`panic!` sobre datos de usuario o fichero en
  `crates/instinct-core/src` y `crates/instinct-cli/src`.
- Sin sitios de llamada sin actualizar para las cinco firmas autorizadas.
- Determinismo: `test_cli_determinism_same_seed` PASS.
- Auditoría estática del diff `38930a2..HEAD`: sin aritmética sin `checked` donde
  había desbordamiento posible, sin casts a `usize` sin comprobar signo
  (`index_from_i64` rechaza negativos con `usize::try_from`), sin iteración
  dependiente de `HashMap`, sin cadenas de `if` por nombre, sin `println!` en
  instinct-core, sin dependencias externas nuevas, sin `f32`/`f64`.

Observación menor (no infringe el contrato): `crates/instinct-core/src/perception.rs:275`
tiene un `.expect("perception resolves")` **dentro de un `#[test]`** en `src/`, sobre
un catálogo por defecto de test. Son datos de prueba, no de usuario ni fichero.

## 5. Desviaciones aplicadas y su justificación

1. **Nombre del ledger**: `NOCHE_F2.md` (runbook §5) en lugar del
   `LEDGER_NOCHE_F2.md` de la ficha genérica. El runbook manda sobre la ficha en lo específico.
2. **Unwrap del stash fugado**: al cierre se descartó `stash@{0}`
   ("fuga-trabajo-paralelo-F2-03") porque el verificador confirmó que contiene
   exactamente las 5 pruebas de F2-06, todas ya presentes y pasando en `develop`.
   Mientras vivió, bloqueó el merge de F2-06 hasta tener el árbol limpio. Se recuperó
   con `git stash push` (reversible), nunca se descartó a ciegas.
3. **Las reanudaciones por `sessionID` no fueron posibles**: el parámetro llega
   truncado a `ses` (probado 4 veces con IDs distintos). Los dos HTTP 429 del proveedor
   (F2-04 y F2-08) se trataron con sesiones nuevas que partían del estado real del
   árbol, registradas como `reanudacion-desviada`. No fueron reintentos a ciegas.
   **Pendiente de revisar en el siguiente runbook**: la política de reanudar la misma
   sesión no es ejecutable con este harness.
4. **F2-08 se rescató sin reimplementar**: el 429 llegó *después* del commit `fcf4c1e`,
   así que no se perdió código; la sesión de cierre solo verificó el commit y completó
   la documentación (`1520f97`).
5. **El agente `reviewer` del runbook §7.5 no existe en el catálogo**: sus funciones de
   auditoría (pánicos, `checked`, casts, `HashMap`, `println!`, tests suavizados, flujos)
   se integraron en el despacho del verificador, que sí está disponible.
6. **`--nologo`**: este cargo lo rechaza en `cargo test`; se ejecuta sin él.
7. **Suciedad inicial**: los materiales de la sesión (plan de F2, `docs/*` nuevos,
   `plan/MASTER.md` replanificado) se comitearon como `38930a2` para arrancar con árbol
   limpio. `plan/F4-arquitectura-patrones/` sigue sin trackear y es ajeno a la sesión.
8. **Orden paralelo**: las tres raíces se cruzaron según disponibilidad. Las únicas
   restricciones de no-solapamiento fueron F2-03 (esperó a F2-01) y F2-04/F2-05
   (ámbitos disjuntos desde el inicio: `interp` vs `labels`/núcleo).

## 6. Incidentes de la sesión

- **Conflicto real de merge** entre F2-03 y F2-06 en `world_tests.rs` (ambos añadían
  pruebas en la misma zona). Resuelto por un subagente de mantenimiento conservando las
  6 pruebas de ambas series; merge `a4d354d`.
- **Fuga de ediciones** al árbol principal durante las sesiones en paralelo (20:21).
  Recuperadas en stash y confirmadas como duplicado; descartadas al cierre.

## 7. Siguiente paso recomendado

La fase F2 queda entregada y aprobada. Recomendado: abrir **F3** (errores y carga
resiliente) con el procedimiento estándar — pre-vuelo sobre `develop`
(`cargo build`/`test`/`clippy`/`fmt`), un runbook propio con el grafo de la fase, y
corregir en esa ficha las dos lecciones de esta: la reanudación por `sessionID` del
harness y la conveniencia de que cada subagente trabaje siempre en un worktree aislado.

## 8. Registro

Trazabilidad completa, evento por evento, en `plan/F2-estabilidad/NOCHE_F2.md`.
