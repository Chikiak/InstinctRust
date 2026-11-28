---
id: G3-01
title: Correcciones priorizadas por evidencia de G2
phase: G3
status: todo
depends_on: [G2-01]
size: L
file: g3-01-correcciones-evidenciadas.md
---

# G3-01 · Correcciones priorizadas por evidencia de G2

## Objetivo

Corregir los defectos que la auditoría de G2-01 (o las notas de sus issues)
haya confirmado, de uno en uno, cada uno con su test de regresión.

## Alcance

**Haz**
- Rama `feat/g3-01-correcciones` desde el tip del stack.
- Leer la matriz y los huecos de G2; elegir por severidad: primero lo que
  rompe una regla del enunciado, después lo que rompe una promesa del núcleo
  (cero pánicos, diagnóstico con línea).
- Por corrección: test que falla antes → fix mínimo → test que pasa después.
  Ramas y PRs separados por corrección si tocan zonas distintas.

**No hagas**
- No corrijas nada que no esté en la lista de evidencia.
- No refactorices de paso (G4 sigue pospuesta).
- No amplíes el banco (eso es G3-02).

### Archivos previstos
- Los que la evidencia determine (candidatos del catálogo: lexer, parser,
  intérprete, mundo — ver PHASE.md de G3).

## Criterios de aceptación
1. Cada corrección con test que falla antes y pasa después (evidencia en el PR).
2. Diff mínimo por corrección; una corrección, un PR.
3. Puertas verdes tras cada merge.

## Verificación
Las tres puertas · `reviewer` auditando cada diff contra la regla afectada.

## Coste y riesgo
Coste: L. Riesgo: tocar el núcleo siempre tiene riesgo de regresión; lo acotan
el test previo y las puertas.
