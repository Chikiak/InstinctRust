---
phase: G3
title: Endurecimiento selectivo
status: todo
master: ../MASTER.md
fuentes:
  - ../docs/PROBLEMAS_Y_SOLUCIONES.md
issues: 2 desglosados en archivos propios (G3-01 a G3-02)
---

# G3 · Endurecimiento selectivo

## Objetivo y entregable

Corregir solo lo que la evidencia de G2 (o un fallo real) haya puesto sobre la
mesa, cada corrección con un test que falla antes y pasa después. El catálogo
de `docs/PROBLEMAS_Y_SOLUCIONES.md` es el menú, no la orden.

## Estado de partida

Verificado el 2026-11-26: catálogo con 35 entradas (L×6, P×7, I×6, W×6, A×5,
C×5) clasificadas en Etapa A (correcciones/features/pruebas) y Etapa B
(patrones). La Etapa B **no se planifica aquí** (ver MASTER, decisión 6).

## Contratos compartidos

- **Un issue por corrección o grupo cohesionado**, con el test que lo demuestra.
- **Candidatos habituales de la Etapa A** (a tomar solo con evidencia): escapes
  `\"`/`\\` en textos, `\r\n` en el lexer, off-by-one de `LINE_LIMIT`, rango de
  `pc` tras `goto`/`resolve_label`, signo antes del cast a `usize` en `Move`,
  `checked_add/sub` en fin de tick.
- **Puertas** por issue, más `reviewer` auditando el diff.

## Issues

| ID | Título | Depende de | Tamaño | Estado |
|----|--------|------------|--------|--------|
| G3-01 | Correcciones priorizadas por evidencia de G2 | G2-01 | L | todo |
| G3-02 | Pruebas allí donde G2 encuentre huecos | G2-01 | M | todo |

### Orden y dependencias

Ambos issues nacen de la matriz de G2-01: su contenido exacto se escribe cuando
la auditoría exista. G3-01 y G3-02 tocan archivos casi disjuntos (código vs.
pruebas) y su orden es libre una vez abiertos.
