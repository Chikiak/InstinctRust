---
id: G0-05
title: Rescate tras la revisión del PR #10 (test del error de colocación y correcciones documentales)
phase: G0
status: done
depends_on: [G0-04]
size: M
file: g0-05-rescate-revision.md
---

# G0-05 · Rescate tras la revisión del PR #10

## Objetivo

Resolver los bloqueantes que el revisor del PR #10 documentó, para que el PR
pueda aprobarse: (1) test del camino de error de `Msg::PrepPlace`, (2) las
correcciones documentales de `plan/G0-convergencia/`, y (3) la restitución del
set de plan nuevo que se perdió en el trasvase de ramas de G0-04.

## Alcance

**Haz**
- Trabajar en el worktree aislado `/home/adrian/Proyectos/wt-g0-fix` (rama
  `feat/g0-05-rescate` sobre el tip de `feat/g0-04-verde-develop`). **Nunca**
  sobre el checkout personal: durante G0-04 el checkout personal quedó cambiado
  de rama y archivos sin commit se perdieron.
- **Código** (`crates/instinct-gui/`): test que ejercite el brazo
  `Err(diagnostic)` de `Msg::PrepPlace` en `app/mod.rs` (colocación fuera de
  rango o celda ocupada): verificar que `placed_count()` no se incrementa y que
  el diagnóstico queda en `log_buffer`. Método: test que falla antes, arreglo
  mínimo si hace falta, test que pasa después.
- **Test existente** (`tests/determinismo/mod.rs`): sustituir el literal `4`
  por `baseline_criaturas().len()` manteniendo una aserción explícita de que el
  baseline tiene 4 elementos.
- **Documentales**: corregir `PHASE.md` (`status: done` → `in_progress` mientras
  queden issues `todo`; rutas `fuentes` a `../../docs/`; typo `dokumentar`),
  `g0-01-notas.md` (conclusión prematura: las puertas se verificaron sobre la
  rama de G0-04, no sobre `origin/develop`; explicar que 397 era un conteo
  truncado por fail-fast —28 suites corrieron frente a las 34 de main—; quitar
  rutas absolutas locales), `g0-04-verde-develop.md` (typo `#[allow]`; «5
  commits» → 6), y newline final en los archivos de `plan/G0-convergencia/`.
- **Plan restituido**: `plan/MASTER.md` (contenido nuevo) y
  `plan/G1-referencia-publicable/`, `plan/G2-conformidad/`,
  `plan/G3-endurecimiento/` — restaurados por el orquestador antes de este issue
  (ver el commit de docs que los contiene).
- Al terminar: push de `feat/g0-05-rescate` a la rama del PR
  (`git push origin feat/g0-05-rescate:feat/g0-04-verde-develop`) para que el
  PR #10 se actualice; NO abras un PR nuevo.

**No hagas**
- No toques `main` ni `origin/develop`.
- No uses `#[allow]` sin razón escrita.
- No vuelvas a cambiar el checkout personal de rama.

### Archivos previstos
- `crates/instinct-gui/tests/**` — nuevo test del camino de error.
- `crates/instinct-gui/tests/determinismo/mod.rs` — modificar — baseline len.
- `plan/G0-convergencia/PHASE.md`, `g0-01-notas.md`, `g0-04-verde-develop.md` — modificar.
- `plan/MASTER.md`, `plan/G1–G3/**` — ya restituidos (solo verificar).

## Criterios de aceptación
1. Existe test que cubre el brazo `Err` de `PrepPlace` (colocación inválida: no
   cuenta y registra diagnóstico); fallaría sin el arreglo de `8dff439`.
2. `cargo test --workspace` exit 0 (461+ tests, 0 fallos).
3. `cargo clippy --workspace --all-targets -- -D warnings` exit 0.
4. `cargo fmt --all --check` exit 0.
5. `PHASE.md` sin contradicción de estado; notas sin afirmación prematura;
   typos y newlines corregidos.
6. Commits sobre la rama del PR (#10) pusheados; árbol del worktree limpio.

## Verificación
Las tres puertas en el worktree · `reviewer` auditando el diff final ·
`git log --oneline origin/develop..HEAD` mostrando los commits propios.

## Coste y riesgo
Coste: M. Riesgo: bajo; el test nuevo puede revelar que el `Err` no se propaga
como se cree —hallazgo bienvenido—.

## Implementation Notes

- **Test nuevo** `crates/instinct-gui/tests/session/state.rs::t6_prep_place_err_no_incrementa_y_guarda_diagnostico`: entra en preparación sobre "Plain with rocks", coloca en celda ocupada `(3, 1)` (roca del mapa) y fuera de rango `(9999, 9999)`, afirma que `placed_count()` no incrementa (se queda en 0) y que `log_buffer` guarda al menos un diagnóstico de tipo `Load`. RED antes del arreglo (`log_buffer` es `None` durante la preparación y el diagnóstico se descarta); GREEN tras el arreglo.
- **Arreglo mínimo** `crates/instinct-gui/src/app/mod.rs`: brazo `Err(diagnostic)` de `Msg::PrepPlace` inicializa `log_buffer` de forma perezosa con `get_or_insert_with` (`Rc<RefCell<LogBuffer>>` con capacidad 4096) y empuja el diagnóstico, en lugar de descartarlo cuando el buffer es `None`. No se toca `start_simulation`, `PrepRemove`, `enum Msg` ni la lógica feliz de `PrepEnter`.
- **Determinismo** `crates/instinct-gui/tests/determinismo/mod.rs`: se sustituye el literal `4` por `baseline_criaturas().len()` en el `assert_eq!` de `placed_count()` y se añade `assert_eq!(baseline_criaturas().len(), 4)` explícito antes del bucle de colocación. `t_determinismo_01_gui_iguala_directo_con_seed7` sigue verde.
- **Puertas**: `cargo test --workspace` (exit 0), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), `cargo fmt --all --check` (exit 0) y smoke camino feliz (`run --root examples --map "Plain with rocks" --place Dwarf@1,1 --ticks 5`, exit 0) — todo verde en el worktree `wt-g0-fix`.
- **Documentales**: ya aplicados en `2d5094a` (orquestador) y verificados: `PHASE.md` con `status: in_progress` y `fuentes → ../../docs/`, `g0-01-notas.md` sin afirmación prematura (verificación sobre la rama de G0-04, 397 truncado por fail-fast), `g0-04-verde-develop.md` con 6 commits, newlines finales en `plan/G0-convergencia/`. No se tocó `main`, `origin/develop`, `plan/MASTER.md` ni `plan/G1-G3/**`.
- **Nota de push**: los commits se dejan locales en `feat/g0-05-rescate` (árbol limpio). El push `git push origin feat/g0-05-rescate:feat/g0-04-verde-develop` para actualizar el PR #10 queda pendiente de autorización humana (no se ejecuta aquí).
