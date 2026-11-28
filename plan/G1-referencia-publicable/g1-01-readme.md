---
id: G1-01
title: README.md de la referencia
phase: G1
status: todo
depends_on: [G0-01, G0-02]
size: M
file: g1-01-readme.md
---

# G1-01 · README.md de la referencia

## Objetivo

Escribir `README.md` en la raíz: qué es InstinctRust, qué crates tiene, cómo
se compila, cómo se corre una simulación desde la terminal, y un ejemplo del
anexo (Hobbit, Ent, Dwarf o Uruk) copiado de una corrida real.

## Alcance

**Haz**
- Rama `feat/g1-01-readme` desde el tip del stack.
- `README.md` con: una línea de qué es; requisitos (toolchain de Rust estable);
  `cargo build --release`; `cargo run -p instinct-cli -- run --root examples
  --map "<mapa>" --place <Species>@<x>,<y> --ticks <n> --seed <s>` explicando
  cada opción; `check <archivo.ins>`; dónde está el banco (`examples/` y sus
  suites); licencia/estado (v0.1.0, referencia en desarrollo).
- Ejemplo ejecutado de verdad: comando y salida copiados de la corrida.
- Si la GUI tiene instrucciones propias (features/eframe), una sección corta
  "ventana" con el comando real de arranque.

**No hagas**
- No inventes salidas: todo bloque de salida sale de una ejecución real.
- No documentes features que no existen en el código.
- No toques `crates/` ni el banco.

### Archivos previstos
- `README.md` — nuevo — documento de entrada de la referencia.

## Criterios de aceptación
1. README en la raíz de main tras el merge.
2. Cada comando del README ejecutado en el worktree de la rama; salidas reales.
3. Un recién llegado (probado por el revisor) puede compilar y correr el ejemplo
   sin información extra.

## Verificación
Puertas verdes · revisión del README contra el código real (subagente `reviewer`).

## Coste y riesgo
Coste: M (escribir y verificar a mano). Riesgo: bajo; el riesgo es documentar
de más, y lo evita el revisor contrastando cada línea con el código.
