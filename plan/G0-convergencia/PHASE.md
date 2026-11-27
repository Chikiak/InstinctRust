---
phase: G0
title: Convergencia main/develop
status: in_progress
master: ../MASTER.md
fuentes:
  - ../../docs/PROBLEMAS_Y_SOLUCIONES.md
issues: 5 desglosados en archivos propios (G0-01 a G0-05)
---

# G0 · Convergencia main/develop

## Objetivo y entregable

Una sola línea de código: `main` conteniendo F6, más la limpieza de `plan/`
ajustada al plan vigente, y una línea base de calidad medida (clippy/fmt).
Entregable observable: PR de `develop` a `main` abierto y revisado; la unión
verificada en worktree aislado con las tres puertas verdes; `plan/` sin las
fases antiguas y con este plan.

## Estado de partida

Verificado el 2026-11-26 sobre el checkout local (`feat/gitignore-plans-local`):

- `origin/main` es ancestro de `origin/develop`: 0 commits propios, 43 detrás
  (todo F6: PRs #1–#7). Fast-forward sin conflictos.
- Checkout de main: `cargo test --workspace` → exit 0 (385 tests, 34 suites).
- Sin PR abiertos; `gh pr list` vacío.
- Ramas locales de limpieza pusheadas sin PR: `feat/limpiar-planes`,
  `feat/gitignore-plans-local` (borran `plan/` y gitignorean `*.local`).
- Camino feliz comprobado: `run --root examples --map "Plain with rocks" --place Dwarf@1,1 --ticks 5`.

## Contratos compartidos

- **Verificación aislada**: toda verificación se hace en un worktree
  (`git worktree add`), nunca sobre el checkout personal, antes de proponer
  cualquier PR.
- **Puertas** (por issue de esta fase y de todas): `cargo test --workspace` exit 0 ·
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0 ·
  `cargo fmt --all --check` exit 0 · smoke del camino feliz.
- **Integración**: solo por PR con `gh`; ningún merge local; la persona revisa y mergea.
- **`plan/`**: tras esta fase contiene únicamente MASTER.md y G0–G3; las fases
  antiguas (F0–F6) se borran, no se conservan en el histórico de main.

## Issues

| ID | Título | Depende de | Tamaño | Estado |
|----|--------|------------|--------|--------|
| G0-01 | Verificar `origin/develop` en worktree y PR develop → main | — | M | todo |
| G0-02 | Ajustar limpieza de `plan/` y `*.local` sobre main convergada | G0-01 | S | todo |
| G0-03 | Línea base de calidad: clippy y fmt medidos sobre main convergada | G0-01 | S | todo |
| G0-04 | Puesta en verde de origin/develop (fmt, clippy, determinismo) | G0-01 | M | done |
| G0-05 | Rescate tras la revisión del PR #10 (test del error de colocación y correcciones documentales) | G0-04 | M | doing |

`G0-04` nació de la medición de G0-01: develop llegó rojo en las tres puertas
(notas en `g0-01-notas.md`), así que el PR a main espera a tener develop verde.
`G0-05` nació de la revisión del PR #10: el revisor pidió un test del camino de
error de `Msg::PrepPlace` y correcciones documentales; el rescate se trabaja en
el worktree `wt-g0-fix`, nunca sobre el checkout personal (que quedó cambiado de
rama durante G0-04).

### Orden y dependencias

G0-01 va primero y desbloquea todo: hasta que main no tenga F6, documentar o
medir calidad sobre main es documentar un proyecto que no existe. G0-02 y G0-03
son independientes entre sí y ambos parten de la main convergada.
