---
id: G1-05
title: Tag v0.1.0 y notas de release
phase: G1
status: todo
depends_on: [G1-01, G1-02, G1-03, G1-04]
size: S
file: g1-05-tag-release.md
---

# G1-05 · Tag `v0.1.0` y notas de release

## Objetivo

Cerrar la fase con una release versionada: tag `v0.1.0` sobre la main verde y
notas que digan qué hay, qué falta y qué se sabe que se desvía.

## Alcance

**Haz**
- Comprobar que `main` tiene mergeados G1-01…G1-04 y las puertas en verde.
- Tag anotado `v0.1.0` y `gh release create v0.1.0` con notas generadas desde
  el contenido real: qué corre hoy (CLI, banco, GUI), línea base de tests,
  limitaciones conocidas (las Desviaciones del banco, si las hay) y hoja vigente.

**No hagas**
- No tagees sobre una main con jobs rojos.
- No inventes notas: cada afirmación sale del repo.

### Archivos previstos
- Ninguno en el árbol de trabajo: tag + release en GitHub.

## Criterios de aceptación
1. Tag `v0.1.0` en el commit de main con todo G1 mergeado.
2. Release publicada con notas verificables contra el repo.
3. Las notas enlazan el plan vigente (`plan/MASTER.md`).

## Verificación
`gh release view v0.1.0` · puertas verdes sobre el commit tageado.

## Coste y riesgo
Coste: S. Riesgo: bajo; el riesgo es publicar de más, y lo evitan las notas
verificables.
