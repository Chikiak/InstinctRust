---
id: G0-02
title: Ajustar limpieza de plan/ y *.local sobre main convergada
phase: G0
status: todo
depends_on: [G0-01]
size: S
file: g0-02-limpiar-plan-local.md
---

# G0-02 · Ajustar limpieza de `plan/` y `*.local` sobre main convergada

## Objetivo

Dejar `plan/` con un único plan vigente (este: MASTER + G0–G3) y los planes
locales gitignoreados, integrado a la main que ya contiene F6.

## Alcance

**Haz**
- Partir de la main convergada (G0-01 mergeado).
- Rama `feat/g0-02-limpiar-plan` desde el tip del stack.
- Commit del plan vigente (`plan/MASTER.md`, `plan/G0-convergencia/`,
  `plan/G1-referencia-publicable/`, `plan/G2-conformidad/`,
  `plan/G3-endurecimiento/`) y borrado de las fases antiguas `plan/F*/`.
- Gitignore de `*.local` y de `plans.local/` (tomar el contenido ya escrito en
  las ramas de limpieza existentes, que quedaron sin PR).
- PR a `main` con `gh`; cuerpo explicando que las ramas de limpieza anteriores
  quedan sustituidas por esta.

**No hagas**
- No borres `plan/` entero: el plan vigente se queda.
- No toques `crates/` ni `docs/`.
- No reescribas el historial de las ramas antiguas.

### Archivos previstos
- `plan/MASTER.md`, `plan/G*/**` — ya escritos, se commitean tal cual.
- `.gitignore` — añadir `*.local` y `plans.local/`.
- `plan/F0/`…`plan/F6*/**` — borrar (llegan con el merge de develop).

## Criterios de aceptación
1. Rama creada desde la main convergada.
2. PR abierto; diff = plan vigente añadido + `plan/F*` borrados + `.gitignore`.
3. `git status` limpio tras el commit; puertas verdes (no cambian código, pero se ejecutan).

## Verificación
`cargo test --workspace` exit 0 · `git status` limpio · diff del PR revisado.

## Coste y riesgo
Coste: S. Riesgo: bajo; el único choque posible sería con contenido nuevo de
`plan/` en develop, resuelto por el orden (G0-01 antes que G0-02).