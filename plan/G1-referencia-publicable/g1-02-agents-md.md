---
id: G1-02
title: AGENTS.md con las reglas del núcleo
phase: G1
status: todo
depends_on: [G0-01]
size: S
file: g1-02-agents-md.md
---

# G1-02 · AGENTS.md con las reglas del núcleo

## Objetivo

Dejar por escrito, para cualquier agente que trabaje en el repo, las reglas
que hoy viven en la memoria del proyecto: restricciones del núcleo, contrato de
diagnósticos, verificación esperada y Convenio de ramas/PRs.

## Alcance

**Haz**
- Rama `feat/g1-02-agents` desde el tip del stack.
- `AGENTS.md` con: workspace de tres crates y qué va en cada uno; núcleo sin
  dependencias externas, sin `f32/f64` (i64 y texto), sin `rand`, sin
  `println!` (eventos), sin pánicos con entrada inválida; diagnóstico único con
  archivo y línea desde 1; entidades por identificador estable; verificación
  (`cargo test --workspace`, clippy, fmt) antes de proponer PR; ramas
  `feat/<slug>` y PR apilado; nada de merges locales; trabajo en worktree
  aislado, nunca sobre el checkout personal (lección del incidente de G0-04:
  el checkout personal quedó cambiado de rama con archivos sin commit).

**No hagas**
- No repitas lo que el README ya dice para humanos.
- No añadas reglas que el código no respeta hoy (verificarlo antes).

### Archivos previstos
- `AGENTS.md` — nuevo — convenciones para agentes.

## Criterios de aceptación
1. `AGENTS.md` en main tras el merge.
2. Cada regla enunciada se verifica contra el código actual (p. ej. `grep` de
   dependencias en `crates/instinct-core/Cargo.toml`).
3. Un agente nuevo que lea solo AGENTS.md puede trabajar sin romper las reglas.

## Verificación
Puertas verdes · revisor contrastando cada regla con el código.

## Coste y riesgo
Coste: S. Riesgo: bajo; documentar una regla que el código viola sería el peor
resultado, y lo evita la verificación del criterio de aceptación 2.
