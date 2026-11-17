---
id: R2-05
title: CLI: separación de streams y validaciones
status: done
phase: R2
depends_on: [R2-01]
size: S
file: r2-05-cli-streams-validation.md
---

# R2-05 · CLI: separación de streams y validaciones

## Leer primero

Este issue revisa la CLI (`crates/instinct-cli/src/`) para confirmar que:

- La salida de simulación (mapa, eventos) va a `stdout`.
- Los diagnósticos y errores van a `stderr`.
- Los flags desconocidos se rechazan.
- `--ticks >= 0`, `--every >= 1`, existencia de `--root` se validan antes de construir.

Mucho de esto ya está hecho en `args.rs` (Builder con validación) y `output.rs`
(`CliOutput` imprime eventos a `stdout`). Este issue verifica que no queden fugas y
que los mensajes de la CLI estén en inglés (tras R2-01).

## Qué revisar

1. **`main.rs`:** usa `build_command` (patrón Command). Los errores van a `stderr`
   vía `eprintln!`. El `--help` va a `stdout`. Verificar que no haya `println!` para
   errores o `eprintln!` para salida de simulación.
2. **`args.rs`:** `parse_run` valida `--ticks >= 0`, `--every >= 1`, existencia de
   `--root`, rechaza flags desconocidos. `RunArgs` se construye solo si todo es válido
   (Builder). Verificar que no falte ninguna validación.
3. **`run.rs`:** `execute_run` imprime diagnósticos de carga a `stderr` (`eprintln!`),
   mapa y eventos a `stdout` (`println!` / `print!` a través del puerto `CliOutput`).
   Verificar que no haya mezclas.
4. **`check.rs`:** `execute_check` imprime `OK` a `stdout` y diagnósticos a `stderr`.
   Verificar.
5. **`command.rs`:** `build_command` rechaza subcomandos desconocidos y `--help` con
   argumentos. Verificar.
6. **Strings en inglés:** tras R2-01, revisar que ningún mensaje de la CLI quede en
   español (`mapas disponibles:` en `run.rs` L30 es un candidato).

## Criterios de aceptación

1. Salida de simulación (mapa, eventos) → `stdout`; diagnósticos/errores → `stderr`.
2. Flags desconocidos rechazados con mensaje claro a `stderr`, salida != 0.
3. `--ticks < 0` rechazado; `--every < 1` rechazado; `--root` inexistente rechazado.
4. Mensajes de CLI en inglés (tras R2-01).
5. `cargo test --workspace` → exit 0 (incluir tests de CLI existentes).
6. `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
7. `cargo fmt --all --check` → exit 0.
8. `git status --short` limpio tras commit.

## Notas de implementación

### Cambios por archivo
- `crates/instinct-cli/src/run.rs` (L30): `eprint!("mapas disponibles: ");` → `eprint!("available maps: ");`. Único cambio de código del issue. Rutas de stdout (L101-108, L120-131) y diagnósticos stderr sin tocar.
- `crates/instinct-cli/tests/cli_run/basic.rs` (`test_cli_unknown_map_exits_1`): se añadió assert de que stderr contiene "available maps" (inglés) tras el assert de "Plain with rocks". TDD: RED antes del cambio, GREEN tras él.
- `plan/R2-refactor-pre-ui/PHASE.md`: fila R2-05 Estado todo→done (tabla Issues) y fila R2-05 rellena en Registro de ejecución.

### Evidencia por AC
- AC1 (simulación→stdout, diagnósticos→stderr): ya OK, sin tocar. `execute_run` imprime mapa y eventos a stdout vía `CliOutput` (run.rs L100-132) y diagnósticos de carga a stderr vía `eprintln!` (run.rs L14-41, L73-88). Cubierto por `test_cli_prints_map` (stderr vacío, mapa en stdout, salida 0).
- AC2 (flags desconocidos →stderr, exit≠0): ya OK, sin tocar. `build_command` en `command.rs` rechaza subcomandos/flags desconocidos con código != 0; cubierto por tests de `validation.rs`.
- AC3 (`--ticks>=0`, `--every>=1`, `--root` existe): ya OK, sin tocar. `parse_run` en `args.rs` valida antes de construir; cubierto por tests de `validation.rs` y `core.rs`.
- AC4 (mensajes en inglés): run.rs:30 traducido + assert en basic.rs. `cargo test -p instinct-cli --test cli_run test_cli_unknown_map_exits_1` → PASS (stderr contiene "available maps").

### Puertas verdes
- `cargo test --workspace` → exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Determinismo `--seed 7`: dos corridas idénticas (SHA256 D6F0D1380CBD856580123B85240855CA0FB547EFCEDA8560BD599EFF7041D6B8 x2), igual al baseline R2-04.
