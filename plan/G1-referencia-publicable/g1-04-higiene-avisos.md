---
id: G1-04
title: Higiene — resolver los avisos medidos en G0-03
phase: G1
status: todo
depends_on: [G0-03]
size: S
file: g1-04-higiene-avisos.md
---

# G1-04 · Higiene: resolver los avisos medidos en G0-03

## Objetivo

Dejar `clippy -D warnings` y `fmt --check` en verde sobre main convergada,
resolviendo exactamente los avisos que la medición de G0-03 lista — ni uno más.

## Alcance

**Haz**
- Rama `feat/g1-04-higiene` desde el tip del stack.
- Resolver cada aviso de `plan/G0-convergencia/g0-03-notas.md` (conocidos de
  antemano: `workspace.package.version` y `workspace.package.edition` sin uso).
- Cambios mínimos y mecánicos; `cargo fmt --all` si hace falta.

**No hagas**
- No arregles "de paso" nada que no esté en la lista de G0-03.
- No refactorices (eso sería G4, y está pospuesto).

### Archivos previstos
- `Cargo.toml` (raíz) — modificar — campos sin uso del workspace.
- Los que la lista de G0-03 añada.

## Criterios de aceptación
1. Cada aviso de las notas de G0-03 resuelto o con justificación escrita de
   por qué no aplica.
2. `clippy -D warnings` y `fmt --check` en verde de principio a fin.
3. Diff mínimo: solo lo que la lista exige.

## Verificación
Las tres puertas · `git diff --stat` acotado a los archivos de la lista.

## Coste y riesgo
Coste: S. Riesgo: bajo; cambios cosméticos, pero las puertas lo confirman.
