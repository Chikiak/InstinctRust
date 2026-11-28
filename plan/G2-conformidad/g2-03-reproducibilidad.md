---
id: G2-03
title: Reproducibilidad con semilla documentada
phase: G2
status: todo
depends_on: [G2-01]
size: S
file: g2-03-reproducibilidad.md
---

# G2-03 · Reproducibilidad con semilla documentada

## Objetivo

Fijar y verificar la promesa de determinismo: misma semilla ⇒ misma simulación,
y dejar el comando de verificación documentado para que cualquiera lo repita.

## Alcance

**Haz**
- Rama `feat/g2-03-determinismo` desde el tip del stack.
- Dos corridas idénticas (mismo `--root`, `--map`, `--place`, `--ticks`,
  `--seed`): diff de salidas vacío.
- Una corrida con semilla distinta: salida diferente (prueba de que la semilla
  manda).
- Documentar en `plan/G2-conformidad/determinismo.md` el procedimiento.

**No hagas**
- No cambies el generador pseudoaleatorio.
- No uses escenarios estadísticos para esta verificación.

### Archivos previstos
- `plan/G2-conformidad/determinismo.md` — nuevo — procedimiento y resultado.

## Criterios de aceptación
1. Dos corridas con la misma semilla producen salidas idénticas (diff vacío).
2. Semilla distinta produce salida distinta en el caso elegido.
3. Procedimiento documentado con los comandos exactos.

## Verificación
Las corridas mismas · puertas verdes.

## Coste y riesgo
Coste: S. Riesgo: bajo; si el determinismo fallara, sería un hallazgo mayor y
se reportaría a la persona de inmediato.
