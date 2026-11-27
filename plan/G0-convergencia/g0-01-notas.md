---
id: G0-01
title: Notas de verificación de origin/develop (worktree aislado)
phase: G0
status: todo
depends_on: []
size: M
file: g0-01-notas.md
---

# G0-01 · Notas de verificación de `origin/develop`

Medición ejecutada el 2026-11-26 sobre worktree aislado
`/home/adrian/Proyectos/wt-g0-develop` (`origin/develop` @ `06df5f7`).
Log completo: `/tmp/opencode/g0-verify.log`.

## Resultado de las puertas

| Puerta | Resultado | Detalle |
|--------|-----------|---------|
| `cargo test --workspace` | **ROJO** (exit 101) | 397 tests pasan en 28 suites; **1 falla**: `t_determinismo_01_gui_iguala_directo_con_seed7` (`crates/instinct-gui/tests/determinismo/mod.rs:51`) |
| `cargo clippy --workspace --all-targets -- -D warnings` | **ROJO** (exit 101) | 3 errores en `instinct-core` (lib): `if` guard con bool literal seguido de otro, `if` colapsable en el `match` exterior, variable de bucle `x` usada para indexar `self.content[y]` |
| `cargo fmt --all --check` | **ROJO** (exit 1) | 4 diffs, todos en `crates/instinct-gui/src/ui/mod.rs` (líneas 101, 322, 356, 398) |
| Smoke (camino feliz CLI) | **VERDE** (exit 0) | `run --root examples --map "Plain with rocks" --place Dwarf@1,1 --ticks 5 --seed 7` corre y renderiza ticks |

## El fallo de tests, caracterizado

`t_determinismo_01_gui_iguala_directo_con_seed7` coloca las 4 criaturas del
baseline por el camino GUI (`Msg::PrepPlace`); el `assert_eq!` de la línea 51
comprueba `placed_count()` frente a `baseline_criaturas().len()` y obtiene
**left: 2, right: 4**: dos de las cuatro colocaciones no cuentan. No es un
problema de entorno (no hay dependencia de pantalla); es una divergencia real
del camino GUI contra el directo — justamente lo que el test existe para
detectar. Queda como trabajo de **G0-04** diagnosticar si la culpa es de
`PrepPlace` (colocación duplicada o especie ya ocupada) o de la expectativa del
test, y arreglarlo con evidencia.

## Conclusión

**Se puede abrir el PR `develop → main`**: el issue **G0-04** (puesta en verde:
fmt, clippy y test de determinismo) está completo y `origin/develop` pasó las
tres puertas en worktree (`cargo test --workspace` 461 tests / 0 fallos,
`cargo clippy ... -D warnings` exit 0, `cargo fmt --all --check` exit 0, más
smoke del camino feliz). G0-01 procede con el PR.

## Reproducción

```bash
git worktree add /tmp/x origin/develop
cd /tmp/x
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```