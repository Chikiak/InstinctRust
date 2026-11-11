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

## Alcance

**Haz**
- Buscar en el repo y en los planes pendientes (F4, F5, F6) si existe un consumidor real para un Visitor sobre el AST: pretty-printer de expresiones, type-checker estático del programa `.ins`, o optimizador de constantes.
- Si hay consumidor real: definir `pub trait ExprVisitor` en `crates/instinct-core/src/expr.rs` con `visit_*` para cada variante (`LiteralInt`, `LiteralText`, `Name`, `Call`, `Unary`, `Binary`), y `Expr::accept(&self, visitor: &mut dyn ExprVisitor)`. Mover la operación del consumidor fuera de `Expr::eval`.
- Si no hay consumidor real (caso esperado): no implementar Visitor. Documentar en las notas de implementación que se descarta por falta de consumidor, citando la decisión 4 de la fase, y dejarlo como candidato para una fase futura.
- En cualquier caso: `Expr` siga siendo Composite (ya aplicado), `eval` siga funcionando, misma semilla = misma historia, cero `unwrap`/`expect`/`panic!` nuevos.

**No hagas**
- No implementes Visitor "por si acaso" sin consumidor (refactor especulativo prohibido por el MASTER, decisión 40).
- No cambies la semántica de evaluación ni el AST (`Expr`, `Line`, `Program`).
- No introduzcas `unwrap`/`expect`/`panic!` sobre datos de entrada.

### Archivos previstos
- `crates/instinct-core/src/expr.rs` — modificar solo si hay consumidor: `ExprVisitor` + `accept`; si no, solo anotar el descarte en comentarios/tests.
- `crates/instinct-core/tests/expr_tests.rs` — modificar — pruebas de `eval` sin cambios (o del Visitor si se implementa).
- `crates/instinct-core/tests/compile_tests.rs` — verificar que la compilación sigue igual.

## Criterios de aceptación

- [ ] **AC1** Se documentó si existe o no un consumidor real de Visitor en este punto del proyecto. Verif: notas de implementación rellenas.
- [ ] **AC2** Si hay consumidor: `ExprVisitor` existe con `visit_*` para cada variante y `Expr::accept` funciona; el consumidor usa el Visitor y no toca los nodos. Verif: `cargo test --workspace`.
- [ ] **AC3** Si no hay consumidor: no hay código Visitor nuevo; el descarte está anotado con motivación y coste. Verif: `cargo test --workspace` en verde (eval igual que antes).
- [ ] **AC4** `Expr::eval` sigue funcionando para todos los nodos; misma semilla = misma historia. Verif: `cargo test -p instinct-core --test expr_tests`.
- [ ] **AC5** Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo. Suite completa + clippy + fmt en verde.

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

Rama: feat/f4-07-visitor-ast (sobre develop). Condicionado por decisión 4 de la fase (PHASE.md, sección "Decisiones de la fase", punto 4): Visitor sobre el AST solo si hay consumidor real.

Descarte: se buscó en repo y planes pendientes (F4, F5, F6) un consumidor real (pretty-printer, type-checker estático, optimizador de constantes). Resultado: no existe. Solo referencias en docs/PATRONES_DISENO_ARQUITECTURA.md (sección 2.2), plan/MASTER.md y los archivos de fase; ningún `ExprVisitor`, impresor ni type-checker en código. Por tanto no se añade código Visitor: sin `trait ExprVisitor`, sin `Expr::accept`, sin cambios en `crates/instinct-core/src/expr.rs` ni en los tests. `Expr::eval` y el AST quedan intactos; misma semilla = misma historia.

Coste asumido: ninguno en código; el patrón queda como candidato para una fase futura cuando aparezca un consumidor real. Motivación: decisión 4 de la fase + MASTER decisión 40 (refactor especulativo prohibido).

Evidencia AC por AC (comando + resultado):
- AC1 PASS: consumidor real inexistente documentado (grep sobre repo: solo docs/planes; ninguna implementación).
- AC2 N/A: no hay consumidor.
- AC3 PASS: sin código Visitor nuevo; descarte anotado con motivación y coste. `cargo test --workspace` → todo ok, 0 fallos.
- AC4 PASS: `cargo test -p instinct-core --test expr_tests` → 10 tests ok, eval igual que antes.
- AC5 PASS: cero unwrap/expect/panic! nuevos; `cargo clippy --workspace --all-targets -- -D warnings` → CLIPPY_OK; `cargo fmt --all --check` → FMT_OK.

Commits: ninguno todavía (pendiente de cierre del issue).