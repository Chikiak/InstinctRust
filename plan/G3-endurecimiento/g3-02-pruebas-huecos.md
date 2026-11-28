---
id: G3-02
title: Pruebas allí donde G2 encuentre huecos
phase: G3
status: todo
depends_on: [G2-01]
size: M
file: g3-02-pruebas-huecos.md
---

# G3-02 · Pruebas allí donde G2 encuentre huecos

## Objetivo

Cerrar los huecos de cobertura que la auditoría de G2-01 haya encontrado,
añadiendo los casos que faltan al banco con su documento de scenario.

## Alcance

**Haz**
- Rama `feat/g3-02-pruebas` desde el tip del stack.
- Por cada hueco de `plan/G2-conformidad/huecos.md`: caso en el banco siguiendo
  la convención de scenario (regla, archivos, comando, resultado documentado en
  `examples/README.md`) más su test en el hogar que corresponda
  (`tests/bank/` para estado interno, `tests/cli_bank/` para lo observable).

**No hagas**
- No toques `crates/*/src/` salvo los `[[test]]` de los `Cargo.toml`.
- No inventes reglas nuevas: los huecos salen del enunciado, no de la
  imaginación.

### Archivos previstos
- `examples/README.md` — modificar — nuevas entradas de scenario.
- `examples/**` — según el hueco (criaturas, mapas, objetos, terrenos).
- `crates/instinct-core/tests/bank/**` y `crates/instinct-cli/tests/cli_bank/**` — nuevos casos.

## Criterios de aceptación
1. Cada hueco cerrado con caso documentado y test verde.
2. Todo caso del README citado por un test que lo verifica.
3. Puertas verdes; `git diff crates/ -- ':!*Cargo.toml'` vacío.

## Verificación
Las tres puertas · revisor contrastando caso ↔ test ↔ regla del enunciado.

## Coste y riesgo
Coste: M. Riesgo: bajo; solo añade datos y pruebas.
