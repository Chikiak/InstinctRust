---
id: F0-01
title: Arnés, diagnósticos y generador sembrado
phase: F0
status: done
depends_on: []
size: S
---

# F0-01 · Arnés, diagnósticos y generador sembrado

## Objetivo
Dejar un workspace de Rust que compila, prueba y pasa lint sin dependencias externas, con los tipos base que usarán todos los issues: el diagnóstico con línea y el generador pseudoaleatorio sembrado. Es la base verificable sin la cual nada posterior se puede comprobar.

## Alcance
**Haz**
- Workspace con los crates `instinct-core` (biblioteca) e `instinct-cli` (binario `instinct-cli`).
- Tipo `Diagnostic` y su formato de texto.
- Tipo `Rng` con semilla, lectura de un entero acotado y barajado de una lista; elige un algoritmo simple y lo anota.
- Copia del enunciado a `docs/enunciado.md` y creación de `docs/ambiguedades.md` con la tabla inicial.
- Binario con uso por defecto y `--help`.

**No hagas**
- Cargar archivos, mundo ni lenguaje (F0-02 en adelante).
- Añadir dependencias externas, incluido el crate `rand`.

### Archivos previstos
- `Cargo.toml` — nuevo — workspace con los dos crates.
- `crates/instinct-core/src/lib.rs`, `diagnostic.rs`, `rng.rs` — nuevos.
- `crates/instinct-cli/src/main.rs` — nuevo — uso y `--help`.
- `crates/instinct-cli/tests/cli_usage.rs` — nuevo.
- `docs/enunciado.md`, `docs/ambiguedades.md` — nuevos.

## Criterios de aceptación
- [ ] **AC1** `cargo build --workspace` termina con exit 0 y `cargo tree --workspace` lista solo los dos crates, sin dependencias externas. Verif: los dos comandos
- [ ] **AC2** `Diagnostic` guarda archivo, línea y mensaje, y se muestra como `archivo:línea: mensaje`. Verif: `test_diagnostic_display_format`
- [ ] **AC3** Con la misma semilla, `Rng` produce la misma secuencia; con semillas distintas, secuencias distintas; el entero acotado entre 0 y 99 nunca sale de ese rango en 10 000 lecturas. Verif: `test_rng_is_deterministic`, `test_rng_range_0_99`
- [ ] **AC4** Barajar una lista con la misma semilla da el mismo orden y conserva todos sus elementos. Verif: `test_shuffle_is_deterministic_permutation`
- [ ] **AC5** `instinct-cli` sin argumentos escribe el uso por stderr y sale con 2; con `--help` lo escribe por stdout y sale con 0. Verif: `test_cli_without_args_exits_2`, `test_cli_help_exits_0`
- [ ] **AC6** `docs/enunciado.md` existe y empieza por la línea `## Instinto`; `docs/ambiguedades.md` contiene una fila por cada ambigüedad de `PHASE.md#contratos-compartidos` (A1 a A13) con columnas decisión, alternativa y "confirmar con profesores". Verif: observación de ambos archivos
- [ ] **AC7** `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` terminan con exit 0. Verif: los tres comandos

## Contexto
**Ya existe**: `plan/MASTER.md` y `plan/F0-esqueleto-andante/PHASE.md`; el enunciado `instinct.md` vive fuera del repo (cópialo desde donde lo tengas).
**Contratos que debes respetar**: `Diagnostic` y `Rng` en `../PHASE.md#contratos-compartidos`; la línea es siempre desde 1.
**Ojo**: el generador debe ser independiente de la plataforma y de la versión de Rust (por eso no se usa `rand`): ninguna operación puede depender del tamaño de `usize`. El test del CLI localiza el binario con la variable de entorno que Cargo ofrece a las pruebas de integración.
**Rust que se practica**: workspaces y módulos, `impl Display`, `Result`, pruebas unitarias y de integración, aritmética con enteros de ancho fijo.

### Leer primero
- `plan/MASTER.md` — decisiones fijadas.
- `plan/F0-esqueleto-andante/PHASE.md` — contratos y tabla de ambigüedades.

## Especificación
- `Diagnostic` se construye con archivo, línea y mensaje, y expone los tres.
- `Rng`: constructor desde una semilla de 64 bits (la semilla 0 debe funcionar); lectura de un entero entre 0 y un máximo inclusive; barajado por el método de Fisher–Yates que usa solo este generador.
- El texto de uso describe los subcomandos `run` y `check`, que se implementan en F0-02 y F0-06; hasta entonces el binario los trata como subcomando desconocido: mensaje por stderr y salida 2.
- En `docs/ambiguedades.md`, la columna "confirmar con profesores" empieza vacía.

## Tests
Unitarios en el núcleo para `Diagnostic` y `Rng`; integración en `crates/instinct-cli/tests/cli_usage.rs` contra el binario real. Sin mocks. Escribe primero los tests del generador.

## Verificación
- `cargo test -p instinct-core` → esperado: tests de AC2 a AC4 en verde.
- `cargo test --workspace` → esperado: sin regresiones, incluidos los del CLI.
- `cargo clippy --workspace --all-targets -- -D warnings` → esperado: exit 0.
- `cargo fmt --all --check` → esperado: exit 0.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si el toolchain no compila el workspace vacío, para y reporta el error exacto; no cambies de edición ni de canal de Rust sin avisar. No decidas tú qué es ambigüedad: usa la tabla de la fase.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

**AC1 PASS** — `cargo build --workspace` exit 0; `cargo tree --workspace` lista solo `instinct-cli` → `instinct-core`, sin dependencias externas. Archivos: `Cargo.toml` (raíz, workspace), `crates/instinct-core/Cargo.toml`, `crates/instinct-cli/Cargo.toml`.

**AC2 PASS** — `Diagnostic` en `crates/instinct-core/src/diagnostic.rs` con `archivo`, `linea`, `mensaje`; `Display` imprime `archivo:línea: mensaje`. Test: `test_diagnostic_display_format` (cargo test -p instinct-core, 8 tests ok).

**AC3 PASS** — `Rng` en `crates/instinct-core/src/rng.rs`, algoritmo SplitMix64 (documentado en comentario de cabecera). Semilla 0 funciona; misma semilla = misma secuencia; semillas distintas = secuencias distintas; 10 000 lecturas `next_bounded(99)` siempre en [0, 99]. Tests: `test_rng_is_deterministic`, `test_rng_range_0_99`, `test_rng_different_seeds_different_sequences`, `test_rng_seed_zero_works`, `test_rng_bounded_zero`.

**AC4 PASS** — `Rng::shuffle` Fisher–Yates in-place; misma semilla = mismo orden; conserva todos los elementos (permutación). Test: `test_shuffle_is_deterministic_permutation`, `test_shuffle_empty_and_single`.

**AC5 PASS** — `crates/instinct-cli/src/main.rs`: sin argumentos → uso por stderr, salida 2; `--help` → uso por stdout, salida 0; subcomando desconocido → stderr + salida 2. Integración: `crates/instinct-cli/tests/cli_usage.rs` con `test_cli_without_args_exits_2`, `test_cli_help_exits_0`, `test_cli_unknown_subcommand_exits_2`. El binario localiza el path con `BIN_INSTINCT_CLI` / `CARGO_BIN_EXE_instinct-cli`.

**AC6 PASS** — `docs/enunciado.md` (copia de `docs/instinct.md`) empieza por `## Instinto`; `docs/ambiguedades.md` tiene una fila por A1–A13 con columnas decisión, alternativa y "confirmar con profesores" (vacía).

**AC7 PASS** — `cargo test --workspace` (11 tests ok), `cargo clippy --workspace --all-targets -- -D warnings` exit 0, `cargo fmt --all --check` exit 0.

Sin `unwrap`/`expect` sobre datos de usuario; sin `rand`; sin `println!` en el núcleo; sin `f32`/`f64`; `usize` solo para índices de slice en `shuffle`, nunca en generación de números.
