---
phase: MASTER
title: Plan general — ciclo de referencia publicable
status: doing
fuentes:
  - ../docs/enunciado.md
  - ../docs/PROBLEMAS_Y_SOLUCIONES.md
  - ../docs/ambiguedades.md
issues: 15 desglosados en archivos propios (G0-01…G3-02, más G0-04 y G0-05)
---

# InstinctRust — Plan general (ciclo 2026-11-26)

## Resumen

- **Objetivo**: convertir el divorcio `main`/`develop` en una sola línea de
  referencia publicable: main convergida con F6, verde y con CI, que cualquier
  profesor pueda clonar y usar para contrastar proyectos de estudiantes.
- **Éxito del ciclo**:
  1. `main` contiene F6 (banco, trampas, rugidos) verificado en worktree aislado antes del PR.
  2. `cargo test --workspace`, `clippy -D warnings` y `fmt --check` verdes en main, con CI que lo exija en cada PR.
  3. README + AGENTS.md + tag `v0.1.0`: la referencia es clonable, comprensible y versionada.
  4. Cada regla de las secciones 2–4 del enunciado tiene un caso del banco con resultado esperado documentado.
- **Punto de partida** (verificado 2026-11-26): 385 tests verdes en 34 suites sobre el checkout de main;
  `origin/main` es ancestro de `origin/develop` (0/43: main sin F6); sin CI, README ni AGENTS.md;
  2 commits de limpieza pusheados sin PR; camino feliz comprobado a mano por CLI.
- **Estrategia**: convergencia primero (G0), luego referencia publicable (G1), después conformidad con el
  enunciado (G2) y endurecimiento selectivo (G3). G4 (patrones de diseño) queda **pospuesta a necesidad real**.

## Alcance

**Incluye**: convergencia de ramas, higiene de release (README/AGENTS.md/CI), auditoría de conformidad
regla-por-regla contra el banco, y las correcciones que esa auditoría evidencie.

**No incluye**: refactors de patrones especulativos (Etapa B del catálogo), extensiones del lenguaje,
máquina de bytecode, sprites, defensa ni evaluación del proyecto.

## Supuestos y decisiones

| # | Supuesto / decisión | Impacto si es falso |
|---|---------------------|---------------------|
| 1 | `develop` puede integrarse a `main` por fast-forward (main es su ancestro) | Habría que resolver conflictos reales en G0-01 |
| 2 | La verificación se hace en worktree aislado, nunca sobre el checkout personal | Un desarrollador trabaja sobre código sucio |
| 3 | La integración es solo por PR con `gh`; ningún merge local | main diverge de otro modo |
| 4 | Las fases antiguas (F0–F6) se borran de `plan/` tras convergir; este plan es el único vigente | Dos planes conviven y se contradicen |
| 5 | La nightly (mandato v1 de instinctrust) solo planifica: 1 plan, 0 código | Código nocturno sin revisión diurna |
| 6 | Las correcciones de G3 salen de la evidencia de G2, no del catálogo completo | Se endurece código que ya funciona y se rompe estabilidad |
| 7 | `cargo test` por defecto es fail-fast: un suite roja trunca el conteo total | Los conteos de tests no son comparables entre corridas |

## Contratos entre fases

- **Verificación por issue** (igual para todos): `cargo test --workspace` exit 0 ·
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0 ·
  `cargo fmt --all --check` exit 0 · `git status` limpio tras el commit · subagente `reviewer` antes de cerrar.
- **Ramas**: un issue = una rama `feat/gX-NN-slug` cortada del tip del stack (PRs apilados), PR a la rama base
  de la fase (`main` salvo G0-01, que es `develop → main`, y los issues de desarrollo sobre `develop`).
- **Definition of Ready** por issue: objetivo medible, alcance acotado, criterios de aceptación ejecutables,
  comandos de verificación y coste estimado.

## Orden

```
G0 (convergencia) → G1 (referencia publicable) → G2 (conformidad) → G3 (endurecimiento selectivo)
G4 (patrones) — pospuesta; solo si G2/G3 la exigen, un patrón por issue
```

## Fases

| Fase | Título | Issues | Depende de |
|------|--------|--------|------------|
| G0 | Convergencia main/develop | G0-01…G0-05 | — |
| G1 | Referencia publicable | G1-01…G1-05 | G0 |
| G2 | Conformidad con el enunciado | G2-01…G2-03 | G1 |
| G3 | Endurecimiento selectivo | G3-01…G3-02 | G2 |

## Historial de cambios del plan

- 2026-11-27: el trasvase de ramas durante G0-04 perdió este archivo y las carpetas G1–G3 (restaurados
  en el PR de G0-05); se añade G0-05 (rescate tras revisión del PR #10) y la decisión 7 sobre fail-fast.
