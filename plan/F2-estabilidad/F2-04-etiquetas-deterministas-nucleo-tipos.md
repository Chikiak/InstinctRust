---
id: F2-04
title: Etiquetas deterministas y pruebas del núcleo de tipos
phase: F2
status: done
depends_on: [F2-02, F2-03]
size: M
---

# F2-04 · Etiquetas deterministas y pruebas del núcleo de tipos

## Objetivo

Cerrar los problemas A4 y A5 y el resto de P7: que la tabla de etiquetas itere en orden
determinista, y que el núcleo de tipos (`Program`, `Value`, `Event`) tenga pruebas propias allí
donde hoy no las tiene. Es el issue que consolida la red de seguridad antes de tocar el intérprete
y el mundo.

## Alcance

**Haz**

- En `crates/instinct-core/src/program.rs`, cambiar `Program::labels` de `HashMap<String, usize>`
  (líneas 73 y 80) a `BTreeMap<String, usize>` para que la iteración sea determinista.
  `resolve_label` conserva su firma y su comportamiento.
- Crear `crates/instinct-core/tests/program_tests.rs` (no existe): `resolve_label` con etiqueta
  conocida y desconocida, y `Line::clone` para cada variante (`Label`, `Goto`, `IfGoto`, `Assign`,
  `Action`) comprobando que el clon conserva expresión, argumentos y línea.
- Ampliar `crates/instinct-core/tests/value_tests.rs` con `to_bool_int` y comparaciones de orden
  con enteros.
- Ampliar `crates/instinct-core/tests/event_tests.rs` con el `Display` de cada variante de `Event`
  (`Said`, `Died`, `RuntimeError`, `Damaged`, `Roared`).
- Cerrar el resto de P7 en `crates/instinct-core/tests/compile_tests.rs`: dos compilaciones del
  mismo texto producen la misma tabla de etiquetas en el mismo orden, y `resolve_label` es estable.
- `cargo test -p instinct-core`, clippy y fmt en verde.

**No hagas**

- No cambies la semántica de `resolve_label` ni de `Line`.
- No toques `lexer.rs`, `expr.rs`, `world.rs`, `sim.rs` ni la CLI.
- No cambies el formato de `Display` de los eventos existentes (solo añade pruebas que lo fijen).

### Archivos previstos

- `crates/instinct-core/src/program.rs` — modificar — `labels` como `BTreeMap`.
- `crates/instinct-core/tests/program_tests.rs` — crear.
- `crates/instinct-core/tests/value_tests.rs` — modificar — cobertura nueva.
- `crates/instinct-core/tests/event_tests.rs` — modificar — `Display` por variante.
- `crates/instinct-core/tests/compile_tests.rs` — modificar — determinismo de etiquetas.

## Criterios de aceptación

- [ ] **AC1** `Program::labels` es un mapa ordenado y su iteración es determinista entre compilaciones del mismo texto. Verif: `test_labels_iteration_deterministic` | `cargo test -p instinct-core --test compile_tests` | prueba pasa.
- [ ] **AC2** `resolve_label` se comporta igual que antes para etiquetas conocidas y desconocidas. Verif: `test_resolve_label_known_and_unknown` | `cargo test -p instinct-core --test program_tests` | prueba pasa.
- [ ] **AC3** `Line::clone` se prueba para las cinco variantes y conserva expresión, argumentos y línea. Verif: `test_line_clone_all_variants` | `cargo test -p instinct-core --test program_tests` | prueba pasa.
- [ ] **AC4** `value_tests.rs` cubre `to_bool_int` y comparaciones de orden con enteros. Verif: `cargo test -p instinct-core --test value_tests` | todas pasan, casos citados en las notas.
- [ ] **AC5** `event_tests.rs` fija el `Display` de cada variante de `Event`. Verif: `cargo test -p instinct-core --test event_tests` | todas pasan.
- [ ] **AC6** Suite completa sin regresiones: `cargo test --workspace` → todo en verde; clippy exit 0; fmt exit 0. Verif: `cargo test --workspace` | `cargo clippy --workspace --all-targets -- -D warnings` | `cargo fmt --all --check`.

## Contexto

**Ya existe**: `Program { lines, labels }` con `labels: HashMap<String, usize>` y `resolve_label`
en `crates/instinct-core/src/program.rs`; `Line` con cinco variantes y un `Clone` manual que usa
`clone_box`; `Value` con conversión a booleano y comparaciones en
`crates/instinct-core/src/value.rs`; `Event` con su `Display` en `crates/instinct-core/src/event.rs`;
las pruebas actuales de `value_tests.rs` y `event_tests.rs`.
**Contratos que debes respetar**: `Program` se construye en `compile.rs` con
`Program { lines, labels }` (línea 365), así que el cambio de tipo de mapa debe compilar allí sin
cambiar comportamiento; `labels` es campo público.
**Ojo**: si algún código itera `labels` hoy, su salida cambiará de orden no determinista a
determinista; eso es la mejora buscada, no una regresión.

### Leer primero

- `crates/instinct-core/src/program.rs` entero (95 líneas).
- `crates/instinct-core/src/value.rs` y `crates/instinct-core/src/event.rs` — variantes a probar.
- `crates/instinct-core/tests/value_tests.rs` y `crates/instinct-core/tests/event_tests.rs` — estilo existente.
- `docs/PROBLEMAS_Y_SOLUCIONES.md` sección 5 (A4, A5).

## Especificación

Un mapa ordenado por nombre de etiqueta, con la misma semántica de consulta. Las pruebas nuevas
son de comportamiento observable: misma entrada, misma salida, y el clon de una línea conserva lo
que la hacía ejecutable.

## Pasos sugeridos (opcional)

1. Cambia el tipo de `labels` y compila: aparecerán los sitios a ajustar.
2. Crea `program_tests.rs` con `resolve_label` y `Line::clone`.
3. Añade las pruebas de `value_tests.rs` y `event_tests.rs`.
4. Añade la prueba de determinismo de etiquetas.
5. `cargo test -p instinct-core` y el gate completo.

## Tests

Nivel: unitario. Casos como comportamientos: etiqueta conocida resuelve al índice correcto,
etiqueta desconocida devuelve `None`, el clon de cada variante ejecuta igual, el `Display` de cada
evento es el esperado, la iteración de etiquetas es estable. Qué NO mockear: `Program` real
compilado con `compile`. Determinismo: es justo lo que se prueba. Datos: programas `.ins` mínimos.

## Verificación

- `cargo test -p instinct-core --test program_tests` → pasan.
- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde + sin stubs/TODOs + sin cambios fuera de alcance.

## Si te bloqueas

Si `BTreeMap` obliga a cambiar un sitio que no es de esta fase, repórtalo en las notas en lugar de
añadir `#[allow]` o una conversión improvisada. Decisiones que NO puedes tomar tú: usar un crate
externo de mapas ordenados (el núcleo no tiene dependencias), o cambiar `resolve_label`.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

Evidencia AC por AC (todas las puertas desde la raíz del worktree, exit 0):

- AC1 PASS: `cargo test -p instinct-core --test compile_tests` → `test_labels_iteration_deterministic` pasa. `Program::labels` es ahora `BTreeMap<String, usize>` (`program.rs` línea 74; `compile.rs` local en línea 61 y construcción `Program { lines, labels }` en línea 392 sin cambio de comportamiento). Tres compilaciones del mismo texto dan iguales claves ordenadas alfabéticamente y `resolve_label` estable (start→0, zeta→1, alpha→3, beta→5, no_existe→None).
- AC2 PASS: `cargo test -p instinct-core --test program_tests` → `test_resolve_label_known_and_unknown` pasa (conocida resuelve al índice correcta; desconocida → `None`; `Program::new()` y `default()` → `None`).
- AC3 PASS: mismo comando → `test_line_clone_all_variants` pasa para las cinco variantes (Label, Goto, IfGoto, Assign, Action); el clon conserva nombre/etiqueta, expresión (`operator` y evaluación con `FakeContext`), argumentos y línea de origen. Se corrigió el acceso al campo `line` de `Line` (enum: no se lee directamente sobre el valor, se desestructura la original y el clon).
- AC4 PASS: `cargo test -p instinct-core --test value_tests` → todas pasan, incluidas `test_to_bool_int` (Int/Text → 0/1) y `test_integer_order_comparisons` (`<`, `<=`, `>`, `>=` con enteros, negativos y cero, sin floats).
- AC5 PASS: `cargo test -p instinct-core --test event_tests` → todas pasan; `Display` fijado por variante: Said, Died, RuntimeError, Damaged, Roared (formato sin cambios, solo pruebas).
- AC6 PASS: `cargo test --workspace` exit 0 (sin regresiones); `cargo clippy --workspace --all-targets -- -D warnings` exit 0 (se eludió la lifetime needless de `find_line` en `program_tests.rs`, sin `#[allow]`); `cargo fmt --all --check` exit 0 (tras `cargo fmt --all`).

Adaptaciones reportadas:
- `interp_tests.rs` no estaba en los archivos previstos pero es sitio de llamada obligado por el cambio de tipo: `Program { labels: ... }` pasó de `HashMap` a `BTreeMap` (línea 267 en mi base).
- Integración de develop: `git merge develop` (fast-forward a4d354d → 49a9804, F2-05 intérprete) tras `git stash -u`; `git stash pop` restauró mi trabajo sin conflictos. `interp_tests.rs` quedó a favor de develop (versión F2-05); sus 4 sitios `labels: HashMap::from(...)` se adaptaron a `std::collections::BTreeMap::from(...)` para compilar contra `labels: BTreeMap<String, usize>` (el de mi stash en la línea 268 + los 3 de develop en líneas 345, 387, 508). Ningún `#[allow]`; cero `unwrap`/`expect` sobre datos de usuario; sin dependencias externas (rand prohibido); sin `println!` en `instinct-core`; sin `f32`/`f64`.

Archivos tocados: `crates/instinct-core/src/program.rs`, `crates/instinct-core/src/compile.rs`, `crates/instinct-core/tests/compile_tests.rs`, `crates/instinct-core/tests/event_tests.rs`, `crates/instinct-core/tests/value_tests.rs`, `crates/instinct-core/tests/interp_tests.rs`, nuevo `crates/instinct-core/tests/program_tests.rs`; plan/F2-estabilidad/F2-04-...md (status done + notas) y plan/F2-estabilidad/PHASE.md (fila done).
