---
id: F5-02
title: Batería de archivos inválidos, uno por error de compilación de 2.8
phase: F5
status: todo
depends_on: [F5-01]
size: M
---

# F5-02 · Batería de archivos inválidos, uno por error de compilación de 2.8

## Objetivo

Tener una carpeta con un archivo `.ins` inválido por cada error de compilación de la sección 2.8,
con su mensaje y línea esperados documentados, y pruebas automáticas que comprueban que cada archivo
produce exactamente ese diagnóstico. Es el entregable observable de la fase: la referencia se puede
auditar error por error sin leer código.

## Alcance

**Haz**
- Carpeta nueva `crates/instinct-cli/tests/fixtures/errors/` con un `.ins` por error de compilación de 2.8 (lista en Especificación).
- `README.md` en esa carpeta con la tabla: archivo → error simulado → mensaje esperado → línea esperada. Es la fuente de la verdad de la batería.
- Suite nueva `crates/instinct-cli/tests/cli_check_errors.rs` que ejecuta `check` sobre cada archivo de la carpeta y comprueba exit code 1, mensaje esperado y línea esperada en stderr.
- Caso de archivo con errores en varias líneas: un diagnóstico por error, ordenados por línea.
- Si algún error de 2.8 no estuviera cubierto por la implementación actual, cubrirlo en el núcleo (el issue lo permite) sin cambiar los mensajes ya existentes.

**No hagas**
- No modifiques los mensajes actuales del compilador salvo para cubrir un hueco real.
- No pongas archivos inválidos en `examples/`: contaminarían las corridas de referencia.
- No cambies la semántica de carga ni la CLI: eso es F5-03.
- No toques errores de ejecución: es F5-04.

### Archivos previstos
- `crates/instinct-cli/tests/fixtures/errors/` — nuevo — un `.ins` por error de 2.8.
- `crates/instinct-cli/tests/fixtures/errors/README.md` — nuevo — tabla error → mensaje → línea.
- `crates/instinct-cli/tests/cli_check_errors.rs` — nuevo — suite que verifica la batería vía la CLI.
- `crates/instinct-core/src/compile.rs` — modificar solo si hay un hueco de cobertura que cubrir.

## Criterios de aceptación

- [ ] **AC1** La carpeta contiene un archivo por cada error de compilación listado en la Especificación y el `README` documenta para cada uno el mensaje y la línea esperados. Verif: listado de la carpeta + lectura del `README`.
- [ ] **AC2** Cada archivo de la batería produce al ejecutar `check` exactamente un diagnóstico con el mensaje y la línea que documenta su `README`, y exit code 1 sin imprimir `OK`. Verif: `cargo test -p instinct-cli --test cli_check_errors`.
- [ ] **AC3** El archivo con varios errores en líneas distintas produce un diagnóstico por error, ordenados por línea ascendente. Verif: `test_multiple_line_errors_are_all_reported` | `cargo test -p instinct-cli --test cli_check_errors`.
- [ ] **AC4** La asignación a percepción y la asignación a constante tienen cada una su archivo y producen su mensaje específico. Verif: `test_cannot_assign_to_perception_or_constant` | `cargo test -p instinct-cli --test cli_check_errors`.
- [ ] **AC5** Los casos de nombre desconocido (acción y función) y de aridad incorrecta (acción y función) tienen archivo propio con mensaje que nombra el elemento y la aridad esperada o recibida. Verif: `test_unknown_name_and_wrong_arity` | `cargo test -p instinct-cli --test cli_check_errors`.
- [ ] **AC6** Los fixtures de `errors/` no afectan a `examples/`: `check` sobre los ejemplos sigue dando `OK`. Verif: `cargo run -p instinct-cli -- check examples\creatures\walker.ins` → `OK`.
- [ ] **AC7** Suite completa, clippy y fmt en verde. Verif: `cargo test --workspace` ; `cargo clippy --workspace --all-targets -- -D warnings` ; `cargo fmt --all --check`.

## Contexto

**Ya existe**: los fixtures sueltos `bad_action.ins`, `bad_header.ins` y `bad_label.ins` en `crates/instinct-cli/tests/fixtures/` con sus casos en `cli_check.rs` (patrón a imitar); `compile.rs` ya emite todos los errores de la lista salvo que el issue encuentre un hueco; `check` ya imprime todos los diagnósticos a stderr y sale con 1.

**Contratos que debes respetar**: diagnóstico con archivo y línea desde 1 (A11: faltas de cabecera se reportan en línea 1); fail-soft no aplica aquí; mensajes existentes sin cambios.

**Ojo**: la batería es un contrato de la fase. Si al escribirla descubres un error de 2.8 sin mensaje o con mensaje que no nombra el elemento, corrígelo en el núcleo dentro de este issue y anótalo en Notas; no lo dejes para después.

### Leer primero
- `crates/instinct-core/src/compile.rs` — los mensajes actuales por tipo de error (cabecera, línea no reconocida, etiquetas, asignación, `Catalog::resolve`).
- `crates/instinct-core/src/expr.rs` — errores del parser de expresiones (`ParseError`).
- `crates/instinct-cli/tests/cli_check.rs` — patrón de `run_cli` y asertos sobre salida y exit.
- `crates/instinct-cli/tests/fixtures/` — los tres fixtures existentes como plantilla de formato.

## Especificación

Errores de compilación de 2.8 y archivo esperado (nombres propuestos, ajustables si el slug choca):

| Error de 2.8 | Archivo | Mensaje esperado (hoy) | Línea esperada |
|---|---|---|---|
| Línea que no encaja en ninguna forma válida | `unrecognized_line.ins` | `unrecognized line: '<texto>'` | la de la línea ofensora |
| Etiqueta usada que no existe | `unknown_label.ins` | `goto to unknown label '<nombre>'` | la del `goto` |
| Etiqueta duplicada | `duplicate_label.ins` | `duplicate label '<nombre>'` | la segunda definición |
| Falta una clave de cabecera | `missing_header_key.ins` | `missing required header key '<clave>'` | 1 (A11) |
| Falta `start:` | `missing_start.ins` | `missing required label 'start'` | 1 |
| Valor inválido en la cabecera | `bad_header_value.ins` | `the value of '<clave>' must be ...` | la del valor |
| Expresión mal formada | `bad_expression.ins` | error del parser de expresiones | la de la expresión |
| Asignación a percepción | `assign_to_perception.ins` | `cannot assign to perception '<nombre>'` | la de la asignación |
| Asignación a constante | `assign_to_constant.ins` | `cannot assign to constant '<nombre>'` | la de la asignación |
| Acción con nombre desconocido | `unknown_action.ins` | `unknown action '<nombre>'` | la de la llamada |
| Acción con aridad distinta | `wrong_action_arity.ins` | la acción espera N argumentos | la de la llamada |
| Función con nombre desconocido | `unknown_function.ins` | función desconocida | la de la llamada |
| Función con aridad distinta | `wrong_function_arity.ins` | la función espera N argumentos | la de la llamada |
| Varios errores, uno por línea | `multiple_errors.ins` | uno por error, ordenados por línea | las de cada error |

Reglas de la batería: cada archivo simula un solo error salvo `multiple_errors.ins`; los mensajes del `README` se copian de la salida real, no se inventan; la suite falla si la salida real se desvía del `README`.

## Tests

Nivel: integración CLI (binario real, como `cli_check.rs`). Casos como comportamientos: "el archivo `unknown_label.ins` sale con 1 y stderr contiene la línea y la etiqueta". No mockees la compilación: `check` debe compilar de verdad. Escribe la suite y el `README` antes de ajustar cualquier hueco del núcleo.

## Verificación

- `cargo test -p instinct-cli --test cli_check_errors` → esperado: un caso por archivo de la batería, todos en verde.
- `cargo run -p instinct-cli -- check examples\creatures\hobbit.ins` → esperado: `OK` (sin contaminación).
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` → esperado: exit 0.
- `cargo fmt --all --check` → esperado: exit 0.
**Terminado** = todos los AC cumplidos con evidencia + comandos en verde.

## Si te bloqueas

Si un error de 2.8 no se puede reproducir con un archivo mínimo, repórtalo en Notas con el motivo; no lo simules con un archivo que produzca otro error. Decisiones que no puedes tomar: cambiar mensajes existentes, mover la batería a `examples/`, tocar la CLI.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: comandos ejecutados y resultado, AC por AC.

### Notas de implementación

<!-- Rama, commit, comandos ejecutados y resultado, AC por AC, huecos cubiertos en el núcleo. -->
