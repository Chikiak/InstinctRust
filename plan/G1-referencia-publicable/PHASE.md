---
phase: G1
title: Referencia publicable
status: todo
master: ../MASTER.md
fuentes:
  - ../docs/enunciado.md
issues: 5 desglosados en archivos propios (G1-01 a G1-05)
---

# G1 · Referencia publicable

## Objetivo y entregable

Que un profesor cualquiera pueda clonar `main`, leer el README, compilar y
correr un ejemplo del anexo sin ayuda; que un agente nuevo respete las reglas
del núcleo sin que nadie se las recuerda (AGENTS.md); y que cada PR futuro
quede bloqueado si rompe tests, lint o formato (CI). Cierre con tag `v0.1.0`.

## Estado de partida

Verificado el 2026-11-26: tras G0, main contiene F6 y `cargo test --workspace`
pasa (el número definitivo de suites se registra en las notas de G0-01). No
existen README.md ni AGENTS.md en ninguna rama; no hay `.github/workflows`.
Avisos conocidos de `cargo`: `workspace.package.version` y
`workspace.package.edition` sin uso.

## Contratos compartidos

- **Puertas** por issue: las tres puertas de MASTER más `git status` limpio.
- **Documentación ejecutable**: todo comando del README se ejecuta de verdad
  antes de escribirlo; el resultado se copia de la corrida real.
- **CI mínima honesta**: test + clippy + fmt; si un paso no puede correr en
  el runner de GitHub (ventana), se marca `continue-on-error` con la razón
  documentada, no se omite en silencio.

## Issues

| ID | Título | Depende de | Tamaño | Estado |
|----|--------|------------|--------|--------|
| G1-01 | README.md de la referencia | G0-01, G0-02 | M | todo |
| G1-02 | AGENTS.md con las reglas del núcleo | G0-01 | S | todo |
| G1-03 | CI en GitHub Actions | G0-03 | M | todo |
| G1-04 | Higiene: resolver los avisos medidos | G0-03 | S | todo |
| G1-05 | Tag v0.1.0 y notas de release | G1-01…G1-04 | S | todo |

### Orden y dependencias

G1-01 y G1-02 son documentación y arrancan en cuanto main está convergada y
limpia de planes viejos. G1-03 y G1-04 esperan la medición de G0-03 (la CI fija
las puertas; la higiene las deja en verde). G1-05 cierra con todo lo anterior
mergeado.
