---
id: R1-01
title: Strings en inglés — causas de muerte, Display de eventos y UI de CLI
phase: R1
status: todo
depends_on: []
size: L
---

# R1-01 · Strings en inglés

## Objetivo

Traducir a inglés los strings observables que quedaron en español: causa de muerte, Display de
eventos y mensajes de la CLI. Cierra las decisiones 1 y 2 de la fase. Es rename puro de strings
(sus aserciones se actualizan en el mismo cambio), no toca lógica.

## Alcance

**Haz**
- `crates/instinct-core/src/world/laws.rs` (~L73): `"vida"` → `"health"`, `"edad"` → `"age"`.
- `crates/instinct-core/src/actions/shared.rs` (~L96): `cause: "vida"` → `"health"`.
- `crates/instinct-core/src/event/display.rs` (~L29): `"… muere por {cause}"` → `"… dies of {cause}"`
  (mantén el formato exacto: tick, especie, @x,y).
- Comentarios/doc que citen `vida`/`edad` como valores: `actions/attack.rs` (L6), `event/events.rs` (L23).
- Mensajes de UI de `crates/instinct-cli/src`: `args.rs`, `check.rs`, `command.rs`, `main.rs`
  (USAGE), `run.rs`. Traduce a mensajes claros en inglés; no cambies el formato de carcasa
  (`--flag`), ni códigos de salida, ni el stream (stdout eventos, stderr diagnósticos).
- Actualiza en el mismo cambio todas las aserciones y expectations afectadas:
  `tests/sim/death.rs` (L54, L100), `tests/actions/attack.rs` (L49),
  `tests/event_tests.rs` (L84-86 y otros Display afirmados),
  `tests/cli_run/f0.rs` (L33, `stdout.contains("muere por vida")`),
  y cualquier otra prueba/fixture que afirme los strings viejos.
- Si `docs/instinct.md` documenta el Display exacto de un evento, actualiza solo esa línea.

**No hagas**
- No cambies ninguna otra palabra de los eventos (`say`, `roar`) ni el idioma de los comentarios
  salvo los citados.
- No traduzcas `plan/` ni `docs/` salvo las líneas de Display afectadas.
- No toques `tests/world/memento.rs` con la variable de programa `vida_extra` (dato de programa,
  no salida): anótalo en "Notas de implementación" como pendiente opcional.

### Archivos previstos
- `crates/instinct-core/src/world/laws.rs`, `actions/shared.rs`, `event/display.rs`,
  `event/events.rs`, `actions/attack.rs` — modificar.
- `crates/instinct-cli/src/args.rs`, `check.rs`, `command.rs`, `main.rs`, `run.rs` — modificar.
- `crates/instinct-core/tests/sim/death.rs`, `tests/actions/attack.rs`, `tests/event_tests.rs`,
  `crates/instinct-cli/tests/cli_run/f0.rs` — modificar (expectations).

## Criterios de aceptación

- [ ] **AC1** `git grep -n '"vida"\|"edad"\|muere' -- crates/` → 0 coincidencias. Verif: comando.
- [ ] **AC2** `git grep -n -iE 'formato|desconocid|directorio|incorrecto|existe' -- crates/instinct-cli/src` → 0 coincidencias en mensajes de UI. Verif: comando.
- [ ] **AC3** Suite completa verde. Verif: `cargo test --workspace`.
- [ ] **AC4** Clippy y fmt. Verif: `cargo clippy --workspace --all-targets -- -D warnings` |
  `cargo fmt --all -- --check`.
- [ ] **AC5** Determinismo intacto: `cargo test -p instinct-core --test annex_examples determinism` y
  las pruebas de `sim/determinism` pasan sin haberlas editado. Verif: comando.

## Contexto

Decisiones del usuario: (1) `vida`/`edad` se traducen, no son spec; (2) strings de UI de CLI se
traducen aunque cambien salida observable. `Display` de `Event` vive en `event/display.rs`; las
causas se emiten desde `world/laws.rs` (fin de tick) y `actions/shared.rs` (muerte por daño).
Las pruebas de E2E afirman strings en stdout (p. ej. `[tick 9] Walker@0,0 muere por vida`).

### Leer primero

`crates/instinct-core/src/event/display.rs`, `world/laws.rs`, `actions/shared.rs`,
`crates/instinct-cli/src/args.rs`, `run.rs`, y las pruebaslistadas en "Archivos previstos".

## Notas de implementación

（vacío tras completar: qué strings quedaron, decisiones, desviaciones）