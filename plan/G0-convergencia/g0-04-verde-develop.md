---
id: G0-04
title: Puesta en verde de origin/develop (fmt, clippy, test de determinismo)
phase: G0
status: done
depends_on: [G0-01]
size: M
file: g0-04-verde-develop.md
---

# G0-04 · Puesta en verde de `origin/develop`

## Objetivo

Dejar `origin/develop` en verde en las tres puertas medidas en las notas de
G0-01, para que el PR `develop → main` integre código sano. Tres trabajos
distintos, cada uno con su commit y su evidencia.

## Alcance

**Haz**
- Base: `develop`. Ramas `feat/g0-04-fmt` y `feat/g0-04-clippy` (mecánicos) y
  `feat/g0-04-determinismo` (conductual), PR a `develop` una por una; o una sola
  rama con tres commits limpios si el revisor lo prefiere — a criterio del
  coordinador, documentado en el PR.
- **fmt**: `cargo fmt --all` — los 4 diffs de `crates/instinct-gui/src/ui/mod.rs`.
- **clippy**: los 3 errores de `instinct-core` (lib) de las notas: `if` guard con
  bool literal, `if` colapsable en `match`, indexado con variable de bucle
  (`x` indexando `self.content[y]`). Corrección mínima; si una requiere cambiar
  comportamiento, se separa en su propio commit con test.
- **determinismo**: diagnosticar por qué `PrepPlace` cuenta 2 de 4 en
  `t_determinismo_01_gui_iguala_directo_con_seed7`
  (`crates/instinct-gui/tests/determinismo/mod.rs:51`). Arreglar la causa con
  test que falla antes y pasa después. Si la expectativa del test es la
  incorrecta, corregirla con la razón escrita en el PR.

**No hagas**
- No refactorices más allá de los tres puntos.
- No toques `main`.
- No uses `#[allow]` para silenciar clippy sin una razón escrita en el PR.

### Archivos previstos
- `crates/instinct-gui/src/ui/mod.rs` — fmt (y quizá clippy).
- El o los archivos de `crates/instinct-core/src/` que correspondan a los 3 lints.
- `crates/instinct-gui/src/app/**` o `crates/instinct-gui/tests/determinismo/mod.rs` — según el diagnóstico.

## Criterios de aceptación
1. `cargo test --workspace` exit 0 sobre `develop` tras los merges (397+ tests, 0 fallos).
2. `cargo clippy --workspace --all-targets -- -D warnings` exit 0.
3. `cargo fmt --all --check` exit 0.
4. El arreglo del determinismo tiene test que falla antes y pasa después.

## Verificación
Las tres puertas en worktree aislado · `reviewer` auditando cada diff.
Re-verificación final registrada como continuación de las notas de G0-01.

## Coste y riesgo
Coste: M. Riesgo: medio-bajo; el punto delicado es el determinismo, que puede
revelar un bug real del camino GUI — hallazgo bienvenido, documentado en el PR.

## Implementation Notes

### Estado de las puertas (verificación final en rama feat/g0-04-verde-develop, cortada de origin/develop)

| Puerta | Comando | Resultado |
|--------|---------|-----------|
| fmt | `cargo fmt --all --check` | exit 0 (4 diffs aplicados en ui/mod.rs) |
| clippy | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| tests | `cargo test --workspace` | exit 0 (461 tests, 0 fallos) |
| smoke camino feliz | `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --place Dwarf@1,1 --ticks 5 --seed 7` | exit 0 |

### Cambios realizados (5 commits de código/test + 1 documental de cierre 7e6a3c0, sin tocar main ni #[allow] sin razón)

1. `fix(core)` — `world/grid.rs`, `compile/assignment.rs`, `interp/visitor.rs`:
   corrige los 3 lints de clippy de instinct-core medidos en G0-01
   (`needless-range-loop` en nearby Pass 2 iterando filas con enumerate,
   `needless-bool` en looks_like_assignment, `collapsible-match` en el
   brazo 'or' del visitor). Semántica inalterada.
   Nota: las referencias a `laws.rs:141` y `snapshot_view.rs:78` del issue no
   se disparan en clippy sobre este checkout (archivos limpios); los errores
   reales son `grid.rs:126`, `assignment.rs:42` y `visitor.rs:55`,
   coincidiendo con `g0-01-notas.md`.
2. `style(gui)` — `ui/mod.rs`: aplica `cargo fmt --all` (4 diffs en líneas
   101, 322, 356, 398), sin tocar lógica.
3. `fix(gui)` — `ui/mod.rs` y `tests/render/snapshot.rs`: literales f32
   (`2.0_f32`, `1.0_f32`) para `float-literal-f32-fallback` y
   `needless-range-loop` en el test de render (necesarios por `--all-targets`).
4. `feat(gui)` — `app/mod.rs`: `Msg::PrepPlace` ya no silencia el `Result` de
   `prep_state.place`; el diagnóstico se propaga al log compartido
   (`push_diagnostic`) y la colocación fallida no se cuenta. Sin tocar
   `PrepEnter`, `start_simulation` ni el enum `Msg`.
5. `test(gui)` — `tests/determinismo/mod.rs`: `PrepEnter` resuelve el mapa
   por nombre "Plain with rocks" en lugar del índice 0 (el índice 0 del
   catálogo alfabético es "Arena", 9x9, que deja 2 colocaciones del baseline
   fuera de rango: `placed_count()==2` RED). Tras el cambio: 4 colocaciones
   y `gui_events == direct_events` GREEN. Aserción explícita
   `placed_count() == baseline_criaturas().len()`. No se tocan `DETERMINISMO_SEED`/`TICKS`,
   `baseline_criaturas`, `common.rs` ni el camino directo (Camino B).

### Archivos no tocados (por decisión del issue)

`main.rs` / `main`, archivos de plan antiguos de `plan/` (F0–F6; su limpieza
corresponde a G0-02), y ningún `#[allow]` sin razón escrita en el PR.

### Commits

- `3d5dcc1 fix(core): clippy needless-range-loop, needless-bool y collapsible-match en instinct-core`
- `3f7eb5a style(gui): apply cargo fmt to ui/mod.rs (4 diffs)`
- `51ca0fb fix(gui): clippy f32 literal fallbacks y needless-range-loop en test de render`
- `8dff439 feat(gui): propagate PrepPlace placement errors instead of silencing them`
- `f9e7c2d test(gui): determinismo prepara sobre "Plain with rocks" y afirma las 4 colocaciones`
