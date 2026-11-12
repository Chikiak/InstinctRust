---
id: F4-07
title: Visitor sobre el AST, solo con consumidor real
phase: F4
status: done
depends_on: []
size: M
---

# F4-07 · Visitor sobre el AST, solo con consumidor real

## Objetivo

Aplicar el patrón Visitor sobre el AST (sección 2.2 de `docs/PATRONES_DISENO_ARQUITECTURA.md`) **solo si llega con un consumidor real** (impresor de expresiones / pretty-printer o type-checker estático). Sin consumidor real, el patrón no entra: se anota como candidato y no se implementa (decisión 4 de la fase). Hoy el AST evalúa con `Expr::eval` dentro del trait `Expr`; separar la estructura de las operaciones permitiría añadir impresión o análisis sin tocar los nodos, pero solo se justifica si hay un consumidor.

## Reapertura (Camino B)

El usuario ha decidido reabrir este issue e implementar el patrón Visitor como mecanismo de recorrido del AST ahora, aunque hoy no haya un consumidor de impresión ni type-checker, para que futuros consumidores (pretty-printer, type-checker, constant folding, linter) lo usen desde el día uno. El parser **no** aplica porque construye nodos en lugar de recorrerlos; el Visitor solo recorre.

Como el alcance antiguo ya no aplica, se sustituye el apartado "Haz" y los Criterios de aceptación por los nuevos de Camino B (decisión explícita del usuario; anotado en las notas de implementación).

## Alcance

**Haz** (Camino B — implementar Visitor)
- Definir `pub trait ExprVisitor` en `crates/instinct-core/src/expr.rs` con `type Output` y `visit_*` para las 6 variantes (`LiteralInt`, `LiteralText`, `Name`, `Call`, `Unary`, `Binary`).
- En `pub trait Expr`: eliminar `fn eval(&self, ctx: &dyn EvalContext) -> Result<Value, String>` y añadir `fn accept(&self, visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>) -> Result<Value, String>;` (`clone_box` y `operator` se quedan igual).
- Implementar `accept` en los 6 nodos como `visitor.visit_xxx(self)` y borrar los `fn eval` de cada nodo. Conservar intactos `clone_box` y `operator`.
- Trasladar el cuerpo de evaluación de cada nodo (mensajes de error exactos y short-circuit de `and`/`or`) tal cual al visitante del intérprete (`InterpreterVisitor` en `interp.rs`), sin cambiar ni una coma de semántica.
- Actualizar los 3 sitios de `execute_line` en `interp.rs`: `expr.eval(ctx)` → `expr.accept(&mut InterpreterVisitor { ctx })`. Confirmar con grep que no quede ningún otro `.eval(` en `src/`.
- Tests TDD: `expr_tests.rs` helper `eval` construye `InterpreterVisitor { ctx }` y llama `accept`; añadir visitor de demostración `NodeCounter` (5 nodos en `"1 + 2 * 3"`); `program_tests.rs`, `functions_tests.rs`, `perception_tests.rs` sustituir `.eval(&ctx)` por `.accept(&mut InterpreterVisitor { ctx })` sin cambiar aserciones.
- Documentar la limitación conocida: `Expr::accept` sirve `Output = Result<Value, String>` porque el árbol es `Box<dyn Expr>` y Rust no permite `accept` genérico sobre objetos de trait; un visitante con otra salida acumula en campos del visitante o se añade otro `accept` a `Expr`.

**No hagas**
- No cambies la semántica de evaluación ni el AST (`Expr`, `Line`, `Program`).
- No introduzcas `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo.
- No dejes stubs ni código muerto: si `eval` queda como rastro en comentarios/doc, debe ser coherente con el nuevo diseño.

### Archivos previstos
- `crates/instinct-core/src/expr.rs` — modificar solo si hay consumidor: `ExprVisitor` + `accept`; si no, solo anotar el descarte en comentarios/tests.
- `crates/instinct-core/tests/expr_tests.rs` — modificar — pruebas de `eval` sin cambios (o del Visitor si se implementa).
- `crates/instinct-core/tests/compile_tests.rs` — verificar que la compilación sigue igual.

## Criterios de aceptación (Camino B)

- [ ] **AC1** Existe `pub trait ExprVisitor` con `type Output` y `visit_*` para las 6 variantes; `Expr::accept` recibe `&mut dyn ExprVisitor<Output = Result<Value, String>>`. Verif: compila + `cargo test -p instinct-core --test expr_tests`.
- [ ] **AC2** El intérprete es un `ExprVisitor` (`InterpreterVisitor` en `interp.rs`) y `Expr` ya NO tiene `eval`. Verif: `grep -n "fn eval" crates/instinct-core/src/expr.rs` sin resultados; `cargo test -p instinct-core --test interp_tests` en verde.
- [ ] **AC3** Semántica de evaluación idéntica: todos los tests de expr/functions/program/perception/interp pasan sin cambios de aserción, incluidos cortocircuito `and`/`or`, división por cero, "unknown variable", "has no creature view". Verif: `cargo test --workspace`.
- [ ] **AC4** Visitante no-intérprete demostrado (`NodeCounter`) y pasando. Verif: `cargo test -p instinct-core --test expr_tests`.
- [ ] **AC5** Determinismo: `test_annex_seed7_sequence_unchanged` + `test_cli_annex_is_deterministic` en verde (sin cambio observable).
- [ ] **AC6** `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` en verde; cero `unwrap`/`expect`/`panic!` nuevos.

## Contexto

**Ya existe**: `pub trait Expr { fn eval(...); fn clone_box(...); fn operator(...) }` con nodos `LiteralInt`, `LiteralText`, `Name`, `Call`, `Unary`, `Binary`; `eval` está dentro del trait (Composite + Interpreter ya aplicados). No hay `ExprVisitor` ni pretty-printer ni type-checker en el repo.

**Contratos que debes respetar**: `Expr::eval` sigue siendo la forma de evaluar; el AST no cambia; cero pánicos en el núcleo.

**Ojo**: la decisión 4 de la fase es explícita — sin consumidor real, el Visitor no entra. No es un fallo del issue: es el resultado correcto. Documenta el descarte y pasa a F4-08.

### Leer primero
- `crates/instinct-core/src/expr.rs` — `trait Expr`, nodos, `eval`, `parse`.
- `crates/instinct-core/tests/expr_tests.rs` — pruebas de evaluación existentes.

## Especificación

```
Si hay consumidor (ej. pretty-printer):
  expr.rs
    trait ExprVisitor {
      fn visit_literal_int(&mut self, v: &LiteralInt);
      fn visit_literal_text(&mut self, v: &LiteralText);
      fn visit_name(&mut self, v: &Name);
      fn visit_call(&mut self, v: &Call);
      fn visit_unary(&mut self, v: &Unary);
      fn visit_binary(&mut self, v: &Binary);
    }
    impl Expr for cada nodo { fn accept(&self, v: &mut dyn ExprVisitor) { ... } }

Si no hay consumidor:
  No se añade nada; solo documentación del descarte.
```

## Tests

Nivel: unitario. Si hay consumidor, prueba el Visitor contra cada nodo; si no, prueba que `eval` sigue igual (no-regresión).

- `expr_tests`: evaluación de todos los nodos igual que antes del issue (o Visitor imprime correctamente).

## Verificación

- `cargo test -p instinct-core --test expr_tests` → evaluación igual que antes.
- `cargo test --workspace` → sin regresiones de F0-F3.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Misma semilla, misma historia (no hay cambio de comportamiento).

**Terminado** = AC1 cumplido (documentado si hay consumidor o no), AC2/AC3 según corresponda, AC4, AC5, comandos en verde + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas

No hay bloqueo técnico: el issue termina con el documento del descarte si no hay consumidor. No forces el Visitor sin consumidor.

## Cierre

- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

## Reapertura (Camino B)

El usuario decidió reabrir el issue e implementar el patrón Visitor como mecanismo de recorrido del AST ahora, para que futuros consumidores (pretty-printer, type-checker, constant folding, linter) lo usen desde el día uno. El parser no aplica porque construye nodos en lugar de recorrerlos; el Visitor solo recorre. Se sustituyó el apartado "Haz" y los Criterios de aceptación por los nuevos de Camino B (decisión explícita del usuario).

Rama: feat/f4-07-visitor-ast (sobre develop). Reapertura expresa de la decisión 4 de la fase: el patrón se aplica aunque hoy no haya consumidor real de impresión/type-checker, porque el intérprete es ya el consumidor real y los futuros consumidores podrán reusar el recorrido.

Archivos tocados:
- `crates/instinct-core/src/expr.rs`: añadido `pub trait ExprVisitor` con `type Output` y `visit_*` para las 6 variantes; `Expr::accept(&self, visitor: &mut dyn ExprVisitor<Output = Result<Value, String>>)` en lugar de `fn eval`; `accept` implementado en los 6 nodos como `visitor.visit_xxx(self)`; borrados los `fn eval` de cada nodo; `clone_box` y `operator` intactos.
- `crates/instinct-core/src/interp.rs`: añadido `pub struct InterpreterVisitor<'a> { pub ctx: &'a dyn EvalContext }` con `impl ExprVisitor` (Output = Result<Value, String>) que copia EXACTAMENTE la lógica de los `eval` (mensajes de error exactos, short-circuit and/or, orden izq→dcha en call); los 3 sitios de `execute_line` usan `expr.accept(&mut InterpreterVisitor { ctx })`. Confirmado con grep: no queda ningún `.eval(` ni `fn eval` en `src/`.
- `crates/instinct-core/tests/expr_tests.rs`: helper `eval` construye `InterpreterVisitor { ctx }` y llama `accept`; añadido `NodeCounter { count: usize }` (Output = Result<Value, String>) que cuenta los 5 nodos de `"1 + 2 * 3"`.
- `crates/instinct-core/tests/program_tests.rs`, `functions_tests.rs`, `perception_tests.rs`: `.eval(&ctx)` → `.accept(&mut InterpreterVisitor { ctx: &ctx })` sin cambiar aserciones.
- `plan/F4-arquitectura-patrones/F4-07-visitor-ast.md`: frontmatter `status: done`; sección "Reapertura (Camino B)"; Haz y Criterios de aceptación sustituidos por los de Camino B.

Limitación conocida (documentada en el issue y en el código): `Expr::accept` sirve `Output = Result<Value, String>` porque el árbol es `Box<dyn Expr>` y Rust no permite `accept` genérico sobre objetos de trait. Un visitante con otra salida (p. ej. `String` para pretty-printer) acumula su resultado en campos del visitante o se añade otro `accept` a `Expr`. Es el coste asumido del patrón en Rust.

Evidencia AC por AC (comando + resultado):
- AC1 PASS: `pub trait ExprVisitor` con `type Output` y 6 `visit_*`; `Expr::accept` recibe `&mut dyn ExprVisitor<Output = Result<Value, String>>`. `cargo test -p instinct-core --test expr_tests` → 11 tests ok.
- AC2 PASS: `grep -n "fn eval" crates/instinct-core/src/expr.rs` → sin resultados. `InterpreterVisitor` en `interp.rs` implementa `ExprVisitor`. `cargo test -p instinct-core --test interp_tests` → 18 tests ok.
- AC3 PASS: `cargo test --workspace` → todo ok, 0 fallos (incluye expr, functions, program, perception, interp; cortocircuito and/or, división por cero, "unknown variable", "has no creature view" sin cambios de aserción).
- AC4 PASS: `cargo test -p instinct-core --test expr_tests` → `test_node_counter_counts_all_nodes` ok, count == 5 para `"1 + 2 * 3"`.
- AC5 PASS: `test_annex_seed7_sequence_unchanged` y `test_cli_annex_is_deterministic` en verde (sin cambio observable).
- AC6 PASS: `cargo clippy --workspace --all-targets -- -D warnings` → exit 0, cero warnings; `cargo fmt --all --check` → exit 0; cero `unwrap`/`expect`/`panic!` nuevos.

Desviaciones: ninguna vs el diseño obligatorio. Se borró el import `Expr` unused en tests (expr_tests, functions_tests, perception_tests) para cumplir `clippy -D warnings`/`fmt --check`.

Commit: `feat: Visitor sobre el AST como mecanismo de recorrido; intérprete como ExprVisitor (F4-07 reapertura, Camino B)` (hash pendiente al cerrar).