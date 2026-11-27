---
id: G0-03
title: Línea base de calidad — clippy y fmt medidos sobre main convergada
phase: G0
status: todo
depends_on: [G0-01]
size: S
file: g0-03-linea-base-calidad.md
---

# G0-03 · Línea base de calidad: clippy y fmt sobre main convergada

## Objetivo

Medir el estado real de lint y formato sobre la main que ya contiene F6, y
dejarlo por escrito como lista de huecos priorizada para G1-04. Si las puertas
ya pasan, el issue lo documenta y no cambia nada.

## Alcance

**Haz**
- Worktree aislado con la main convergada.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check`.
- Registrar en `plan/G0-convergencia/g0-03-notas.md`: resultado, y si hay avisos,
  la lista exacta (archivo:línea, mensaje) ordenada por frecuencia.
- Si todo está verde: documento de una línea confirmándolo.

**No hagas**
- No corrijas los avisos en este issue (eso es G1-04).
- No toques el checkout personal.

### Archivos previstos
- `plan/G0-convergencia/g0-03-notas.md` — nuevo — resultado de la medición.

## Criterios de aceptación
1. Ambas puertas ejecutadas sobre main convergada en worktree aislado.
2. Notas escritas con el resultado exacto de cada puerta.
3. Si hay avisos: lista priorizada, cada uno con sitio y motivo, lista para G1-04.

## Verificación
Las dos puertas, más `git status` que muestre solo el archivo de notas nuevo.

## Coste y riesgo
Coste: S (recompilación incremental en worktree). Riesgo: ninguno; es medición.