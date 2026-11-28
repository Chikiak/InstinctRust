---
id: G0-01
title: Verificar origin/develop en worktree aislado y abrir PR develop → main
phase: G0
status: todo
depends_on: []
size: M
file: g0-01-verificar-develop-y-pr.md
---

# G0-01 · Verificar `origin/develop` y PR develop → main

## Objetivo

Comprobar, sobre un worktree aislado con `origin/develop`, que la línea con F6
pasa las tres puertas y el smoke; después abrir el PR `develop → main`
(`gh pr create --base main --head develop`) con la evidencia en el cuerpo.

## Alcance

**Haz**
- Worktree aislado: `git worktree add /tmp/opencode/g0-develop origin/develop`
  (o ruta equivalente fuera del checkout personal).
- En el worktree: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo fmt --all --check`, y el smoke del camino feliz con un escenario del banco
  (p. ej. `run --root examples --map "Plain with rocks" --place Dwarf@1,1 --ticks 5`).
- PR con `gh` de `develop` a `main` con título y cuerpo: qué integra (F6-01…F6-06),
  evidencia de las puertas y enlace al plan (`plan/G0-convergencia/PHASE.md`).

**No hagas**
- No trabajes sobre el checkout personal.
- No hagas merge local ni toques `main`/`develop` con commits directos.
- No corrijas código: si algo falla, el issue termina con el fallo documentado.

### Archivos previstos
- Ninguno en el repositorio de trabajo: este issue solo verifica y abre PR.

## Criterios de aceptación
1. Worktree creado desde `origin/develop` con árbol limpio.
2. `cargo test --workspace` exit 0 en el worktree; número de suites y tests registrado.
3. `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` exit 0,
   o lista exacta de avisos reales si los hubiera (entrada de G0-03).
4. Smoke del camino feliz con salida documentada.
5. PR `develop → main` abierto con la evidencia en el cuerpo.

## Verificación
Las cuatro puertas arriba, ejecutadas íntegras en el worktree (sin `--filter`).

## Coste y riesgo
Coste: una compilación completa del worktree (dependencias de GUI incluidas).
Riesgo: bajo mecánicamente (fast-forward); el riesgo real es descubrir F6 rojo,
en cuyo caso el PR no se abre y el fallo se reporta a la persona.
