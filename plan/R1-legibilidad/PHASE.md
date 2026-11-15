---
phase: R1
title: Legibilidad y organización (refactor no planificado)
status: planned
master: ../MASTER.md
fuentes: ../../.opencode/ledgers/refactor-recon-report.md
issues: 9 desglosados en archivos propios (R1-01 a R1-09)
---

# R1 · Legibilidad y organización (refactor no planificado)

## Objetivo y entregable

Cerrar la deuda de legibilidad detectada tras F4: separar archivos-dios, terminar de splitear las
pruebas planas que quedaron, homogeneizar comentarios (concisos, explicativos, en inglés) y
homogeneizar strings de dominio y de UI en inglés. **Sin cambios de lógica**: solo mover, separar,
renombrar strings y reescribir comentarios.

Entregable observable: mismo comportamiento (salvo strings renombrados por decisión), cero
warnings, cero comentarios en español en `crates/`, y ningún archivo de `crates/` por encima de
~300 líneas salvo justificación escrita en el issue.

## Estado de partida

- Rama `develop` limpia; `cargo clippy --workspace --all-targets -- -D warnings` PASS;
  `cargo fmt --all -- --check` PASS; `cargo test --workspace` EXIT=0.
- Split previos ya fusionados: `cli_run`, `program_tests`, `lexer_tests`, `resources_tests`,
  fix de imports de clippy. `src/` y la mayoría de `tests/` ya están modularizados.
- Auditoría read-only (subagentes, ver `.opencode/ledgers/refactor-recon-report.md`) con la deuda
  restante priorizada: god files (`world/creature.rs` ~530, `perceptions/builtins.rs` ~470+,
  `compile/body.rs` ~330), duplicación en `resources/folder.rs`, pruebas planas sin splitear
  (`rng_tests`, `event_tests`, `value_tests`, `species_tests`, `diagnostic_tests`), tests largos
  en 8 módulos ya-split, pruebas planas de `instinct-cli` (`cli_annex`, `cli_check`, `cli_usage`),
  strings en español (`vida`/`edad` como causa de muerte, Display de eventos, UI de CLI) y
  artefactos trackeados en la raíz (`*_out.txt`).

## Contratos compartidos

- **Cero cambios de lógica.** Solo: mover código entre módulos, dividir archivos/tests, renombrar
  strings (con sus aserciones afectadas en el mismo cambio), reescribir comentarios. Nada de
  reordenar turnos, cambiar fórmulas, límites o semillas.
- **Puerta de salida por issue**: `cargo test --workspace` verde ·
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0 ·
  `cargo fmt --all -- --check` exit 0 · `git status` limpio tras el commit.
- **Determinismo intacto**: las pruebas de determinismo existentes (`sim/determinism`,
  `annex_examples/determinism`, E2E con semilla fija) siguen pasando sin tocarlas.
- **API pública del núcleo**: si un split cambia la ruta de un módulo, se reexporta desde el
  `mod.rs` para que ningún consumidor (CLI, tests) necesite cambios salvo los imprescindibles, y
  esos se actualizan en el mismo commit.
- **Comentarios**: inglés, concisos, explican el porqué; se borran los que solo releen el código.
- **Objetivo de tamaño**: ~300 líneas por archivo como techo blando; el criterio real es
  responsabilidad única y legibilidad.

## Decisiones de la fase

1. **Strings `vida`/`edad` (causa de muerte) se traducen**: `"vida"` → `"health"`, `"edad"` →
   `"age"`; Display de eventos `"… muere por …"` → `"… dies of …"`. No son parte del spec.
2. **Strings de UI de la CLI se traducen** al inglés (cambian salida observable por decisión
   explícita del usuario).
3. **El techo de ~300 líneas es orientativo**: manda la cohesión por responsabilidad; dividir por
   dividir sin mejorar organización está prohibido.

## Issues

Desglosado en 9 archivos de issue en `plan/R1-legibilidad/`, listos para ejecutar en orden.
Estado inicial: todos `todo`.

| ID | Título | Cubre | Depende de | Tamaño | Estado | Archivo de issue |
|----|--------|-------|------------|--------|--------|------------------|
| R1-01 | Strings en inglés: causas de muerte, Display de eventos y UI de CLI | Renames `vida`/`edad`, `muere por`, mensajes de CLI | — | L | done | r1-01-english-strings.md |
| R1-02 | Dividir `world/creature.rs` | God file (~530) | R1-01 | L | done | r1-02-split-world-creature.md |
| R1-03 | Dividir `perceptions/builtins.rs` | 25 structs de percepción cuasi-clones | — | L | done | r1-03-split-perceptions-builtins.md |
| R1-04 | Dividir `compile/body.rs` | Mezcla de fases en `compile()` | — | M | done | r1-04-split-compile-body.md |
| R1-05 | `resources/folder.rs` genérico y limpieza de comentarios | 3 loaders cuasi-duplicados; comentarios obvios | R1-01 | S | todo | r1-05-folder-loader-y-comentarios.md |
| R1-06 | Splitear pruebas planas de instinct-core | `rng_tests`, `event_tests`, `value_tests`, `species_tests`, `diagnostic_tests` (+ comentarios ES) | R1-01 | M | todo | r1-06-split-core-tests-planos.md |
| R1-07 | Partir tests largos en módulos ya-split | 8 módulos con tests de 60-140 líneas | R1-06 | L | todo | r1-07-partir-tests-largos.md |
| R1-08 | Splitear pruebas de instinct-cli y deduplicar helpers | `cli_annex`, `cli_check`, `cli_usage` planos | R1-01 | M | todo | r1-08-split-cli-tests.md |
| R1-09 | Higiene del repo: artefacts `*_out.txt` fuera de git | Artefactos trackeados en la raíz | — | S | todo | r1-09-higiene-repo-artefactos.md |

### Orden y dependencias

- R1-01 va primero: renombra strings que varios archivos de tests afirman; hacerlo antes de mover
  tests evita solapes.
- R1-02, R1-03, R1-04, R1-05, R1-08, R1-09 son independientes entre sí (se ejecutan en serie igual).
- R1-06 precede a R1-07 (partir los archivos planos antes de partir tests dentro de carpetas ya
  split para no tocar dos veces el mismo árbol).

## Verificación de salida de la fase

- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all -- --check` → exit 0.
- `git grep` de comentarios en español sobre `crates/` → 0 coincidencias (decisiones 1 y 2).
- Ningún archivo `.rs` de `crates/` por encima de 300 líneas salvo excepción anotada en su issue.
- Determinismo: las pruebas con semilla fja existentes pasan sin modificarlas.
- `git status` limpio; sin `*_out.txt` trackeados (R1-09).

## Registro de ejecución

| Issue | Motivación | Alcance real | Estado |
|-------|------------|--------------|--------|
| R1-01 | Strings en español en salida observable | — | done |
| R1-02 | `creature.rs` mezcla tipos + mundo + leyes | `creature/types.rs`, `creature/mod.rs`, `world.rs`, `life.rs`, `ticks.rs`; `creature.rs` borrado; `mod.rs` y `clone_debug.rs` ajustados | done |
| R1-03 | Monotonía de structs en un solo archivo | `builtins/core.rs` (Health, Age, Lifespan, X, Y, Tick, Vision), `builtins/near.rs` (*Near, *Dist, *Dx, *Dy), `builtins/social.rs` (especie, facción, terreno, reserva, totales, roars, random), `builtins/mod.rs` (reexports); `builtins.rs` borrado. `perceptions/mod.rs` y `catalog.rs` sin cambios. 31 nombres reexportados; 0 cambios de lógica; todo < 300 líneas. | done |
| R1-04 | `compile()` mezcla fases | `body.rs` <150 líneas; cabecera en `header.rs` (`process_header_line`), despacho en `dispatch.rs` (`dispatch_body_line`), post-validación en `validate.rs` (`finalize_validation`); `compile()` orquestador sobre `CompileState`; firma y orden de diagnósticos intactos; 0 cambios de lógica | done |
| R1-05 | Carga duplicada por tipo de recurso | — | todo |
| R1-06 | Pruebas planas sin patrón de carpeta | — | todo |
| R1-07 | Tests de 60-140 líneas ilegibles | — | todo |
| R1-08 | Pruebas CLI planas, helpers duplicados | — | todo |
| R1-09 | Artefactos versionados | — | todo |