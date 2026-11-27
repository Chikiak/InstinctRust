---
id: G2-02
title: Los cuatro ejemplos del anexo, de principio a fin
phase: G2
status: todo
depends_on: [G2-01]
size: M
file: g2-02-anexo-ejemplos.md
---

# G2-02 · Los cuatro ejemplos del anexo, de principio a fin

## Objetivo

Verificar que los cuatro ejemplos que el enunciado cita (Hobbit, Ent, Dwarf y
el Uruk de la sección 2.1) cargan, compilan y corren con salida documentada —
son los primeros que un profesor probaría.

## Alcance

**Haz**
- Rama `feat/g2-02-anexo` desde el tip del stack.
- Por cada ejemplo: `check` del `.ins`, carga con `run` sobre un mapa del
  banco, salida copiada de la corrida real.
- Documentar en `plan/G2-conformidad/anexo.md`: comando exacto, semilla, salida
  observada y resultado esperado (o la desviación encontrada).

**No hagas**
- No ajustes criaturas ni mapas para que "cuadren": si un ejemplo falla, se
  documenta la desviación y se reporta a G3-01.
- No inventes salidas.

### Archivos previstos
- `plan/G2-conformidad/anexo.md` — nuevo — los cuatro ejemplos verificados.

## Criterios de aceptación
1. Los cuatro ejemplos ejecutados en el worktree de la rama.
2. `check` y `run` por ejemplo, con salida real copiada.
3. Cada desviación anotada con su regla del enunciado afectada.

## Verificación
Puertas verdes · salidas reproducibles (misma semilla, misma salida).

## Coste y riesgo
Coste: M. Riesgo: bajo; documentar una desviación es el resultado válido.
