---
phase: G2
title: Conformidad con el enunciado
status: todo
master: ../MASTER.md
fuentes:
  - ../../docs/enunciado.md
  - ../../docs/ambiguedades.md
issues: 3 desglosados en archivos propios (G2-01 a G2-03)
---

# G2 · Conformidad con el enunciado

## Objetivo y entregable

La razón de ser del proyecto: ser referencia frente a proyectos de estudiantes.
Entregable observable: una matriz regla → caso del banco → resultado documentado
para las secciones 2–4 del enunciado, con los huecos convertidos en issues, y
los cuatro ejemplos del anexo corriendo de principio a fin con semillas fijas.

## Estado de partida

Verificado el 2026-11-26: banco (F6) integrado en main tras G0-01; suites
`crates/instinct-core/tests/bank/` y `crates/instinct-cli/tests/cli_bank/`
presentes; `examples/README.md` documenta scenarios con su regla; las
ambigüedades resueltas están en `docs/ambiguedades.md` (A1–A13, algunas sin
confirmar con los profesores).

## Contratos compartidos

- **Regla del enunciado**: unidad de auditoría. Se cita por sección y número
  (p. ej. «2.4 move», «3.1 terreno»). Cada regla debe tener al menos un caso.
- **Resultado esperado = comportamiento de esta referencia**, obtenido
  ejecutando el caso. Si contradice el enunciado, se anota como desviación y se
  reporta a G3-01; **no se corrige el núcleo durante la auditoría**.
- **Determinismo**: todo caso con semilla fija; los que dependen de `random`
  se marcan estadísticos.

## Issues

| ID | Título | Depende de | Tamaño | Estado |
|----|--------|------------|--------|--------|
| G2-01 | Auditoría regla-por-regla contra el banco | G1-01 | L | todo |
| G2-02 | Los cuatro ejemplos del anexo, de principio a fin | G2-01 | M | todo |
| G2-03 | Reproducibilidad con semilla documentada | G2-01 | S | todo |

### Orden y dependencias

G2-01 es el carné de identidad del proyecto y alimenta a los demás: primero la
matriz completa, que ya dice qué falta. G2-02 verifica los ejemplos citados por
el enunciado (los que un profesor usaría primero). G2-03 fija la reproducibilidad
que la auditoría necesita para ser repetible.
