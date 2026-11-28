---
id: G2-01
title: Auditoría regla-por-regla contra el banco
phase: G2
status: todo
depends_on: [G1-01]
size: L
file: g2-01-auditoria-reglas.md
---

# G2-01 · Auditoría regla-por-regla contra el banco

## Objetivo

Recorrer las secciones 2–4 del enunciado y, por cada regla, comprobar si existe
un caso del banco que la verifique con resultado documentado. Producto: matriz
completa y lista de huecos como issues nuevos.

## Alcance

**Haz**
- Rama `feat/g2-01-auditoria` desde el tip del stack.
- Leer `docs/enunciado.md` secciones 2–4 y extraer la lista de reglas (vida y
  ticks, movimiento y bloqueos, ataques, consumo, reproducción, muerte por
  edad, percepciones, expresiones y tipos, rugidos/say, carga tolerante,
  fin de tick, regeneración de terrenos, objetos agotados…).
- Para cada regla: localizar el caso en `examples/README.md` y su test
  (`tests/bank/`, `tests/cli_bank/`). Ejecutarlo. Anotar en la matriz: regla,
  caso, test, resultado observado, conforme/desviación.
- Huecos: cada regla sin caso se anota como issue nuevo en
  `plan/G2-conformidad/huecos.md` con la forma que tendría el caso (no se
  implementa aquí).

**No hagas**
- No corrijas el núcleo ni amplíes el banco: este issue mide, no arregla.
- No des por bueno un caso sin ejecutarlo.

### Archivos previstos
- `plan/G2-conformidad/matriz-conformidad.md` — nuevo — la matriz completa.
- `plan/G2-conformidad/huecos.md` — nuevo — issues candidatos por hueco.

## Criterios de aceptación
1. Toda regla de las secciones 2–4 con fila en la matriz (regla, caso, test,
   resultado, estado).
2. Cada caso afirmado en la matriz fue ejecutado en el worktree de la rama.
3. Huecos listados con la forma del caso que faltaría, listos para convertirse
   en issues.

## Verificación
Puertas verdes · revisor muestreando filas de la matriz contra el repo
(caso existe, test existe).

## Coste y riesgo
Coste: L. Riesgo: descubrir desviaciones reales de la referencia frente al
enunciado — no es un riesgo del issue, es su propósito: quedan documentadas y
alimentan G3-01.
