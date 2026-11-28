---
id: G0-07
title: Rescate 2 tras la segunda revisión del PR #10 (test t6 laxo, rutas de fuentes, notas del PR)
phase: G0
status: doing
depends_on: [G0-05]
size: M
file: g0-07-rescate-revision-2.md
---

# G0-07 · Rescate 2 — bloqueantes de la segunda revisión del PR #10

## Objetivo

Resolver los 4 bloqueantes que el revisor documentó en su segunda revisión del
PR #10 (veredicto `Request changes`), más las mejoras menores, y dejar el PR
actualizado: rama pusheada, descripción corregida y salidas de `git log` /
`git cherry` / puertas en el hilo.

## Contexto

- Trabajo en el worktree aislado `/home/adrian/Proyectos/wt-g0-fix`
  (rama `feat/g0-05-rescate`, tip `e558656`). **Nunca** sobre el checkout
  personal (que quedó cambiado de rama y perdió archivos sin commit en G0-04).
- La subida a `origin/feat/g0-04-verde-develop` (rama del PR #10, sin PR nuevo)
  está autorizada por la persona para el alcance del rescate.
- No tocar `main` ni `origin/develop`. Sin merges locales.

## Alcance

**Haz**

1. **Código — `crates/instinct-gui/tests/session/state.rs`,
   `t6_prep_place_err_no_incrementa_y_guarda_diagnostico` (líneas ~126–152):**
   el test ejercita dos rechazos distintos (celda ocupada `(3,1)` por roca y
   fuera de rango `(9999,9999)`) pero solo comprueba `!entries.is_empty()` y
   que alguna entrada sea `LogEntryKind::Load`. Endurecerlo: verificar la
   longitud exacta (`entries.len() == 2`, o `len() == 1` tras el primer
   `PrepPlace` y `== 2` tras el segundo) y que **ambos** diagnósticos sean
   `Load` con texto no vacío y contexto distintivo (ocupada vs fuera de rango).
   Debe seguir siendo RED sin `99b5dbe`.
2. **Docs — rutas de `fuentes`**: en `plan/G1-referencia-publicable/PHASE.md`
   (línea 7), `plan/G2-conformidad/PHASE.md` (líneas 7–8) y
   `plan/G3-endurecimiento/PHASE.md` (línea 7): `../docs/` → `../../docs/`.
3. **Docs — `plan/G0-convergencia/g0-04-verde-develop.md`**: línea 72 dice
   «6 commits» pero `### Commits` lista 5 (falta `7e6a3c0`): alinear a
   «5 commits de código/test + commit documental de cierre `7e6a3c0`»; línea 96
   dice `placed_count()==4` → `placed_count() == baseline_criaturas().len()`
   (la aserción explícita de longitud 4 llegó en G0-05 con `0d7264b`).
4. **Docs — notas de push**: en `g0-05-rescate-revision.md`, quitar la nota de
   «push pendiente» (el push a la rama del PR ya se ejecutó) y registrar los
   hashes finales.
5. **Menores**: verificar que `start_simulation` no sobrescriba
   incondicionalmente `self.log_buffer` (conservar diagnósticos de preparación
   tras Play) y que `Msg::PrepEnter` lo limpie al cambiar de mapa si procede;
   añadir `assert!(grid.cells[1].len() >= 3)` (o equivalente) en
   `tests/render/snapshot.rs` (`fila_solo_terreno_todas_celdas_iguales`);
   typos: `recuerda`→`recuerde` (G1/PHASE.md:16), `cambiarado`→`cambiado` y
   «del aceptación 2»→«del criterio de aceptación 2» (g1-02-agents-md.md),
   unir la viñeta partida en `g0-01-verificar-develop-y-pr.md`, y
   `criaturos`→`criaturas` si aparece en los docs.
6. **Cierre**: ejecutar las tres puertas y el smoke en el worktree, hacer
   commits modulares (uno por cambio cohesionado; código y docs separados),
   pushear a `origin/feat/g0-04-verde-develop`, y dejar en el hilo del PR la
   descripción corregida (la de la sección 1 de la revisión) más las salidas
   de `git log --oneline --graph origin/develop..HEAD` y
   `git cherry -v origin/develop`.

**No hagas**

- No abras un PR nuevo ni toques `main`/`origin/develop`.
- No uses `#[allow]` ni debilites aserciones existentes.
- No trabajes sobre el checkout personal.

## Criterios de aceptación
1. `t6` verifica los 2 diagnósticos (longitud exacta + contenido), RED sin `99b5dbe`.
2. `cargo test --workspace` exit 0 (462+ tests, 0 fallos).
3. `cargo clippy --workspace --all-targets -- -D warnings` exit 0.
4. `cargo fmt --all --check` exit 0.
5. `fuentes` con `../../docs/` en G1/G2/G3; notas de G0-04 alineadas.
6. Rama del PR actualizada y evidencias pegadas en el hilo.

## Verificación
Las tres puertas + smoke en el worktree · `reviewer` auditando el diff final.