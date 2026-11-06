//! Unit tests para el núcleo de tipos compilados: `Program`, `Line` y la
//! tabla de etiquetas (`labels`).
//!
//! Los programas se construyen de verdad con `compile`; no se mockea el
//! núcleo.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::expr::{Expr, FakeContext};
use instinct_core::perception::PerceptionCatalog;
use instinct_core::program::{Line, Program};
use instinct_core::value::Value;

fn default_catalogs() -> (Catalog, PerceptionCatalog) {
    (Catalog::default(), PerceptionCatalog::default())
}

fn compile_text(text: &str) -> instinct_core::compile::CompileResult {
    let (cat, pc) = default_catalogs();
    compile::compile(text, &cat, &pc, "test.ins")
}

// ---------------------------------------------------------------------------
// AC2: resolve_label con etiqueta conocida y desconocida
// ---------------------------------------------------------------------------

#[test]
fn test_resolve_label_known_and_unknown() {
    let text =
        "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\ngoto end\nend:\n";
    let result = compile_text(text);
    assert!(
        result.diagnostics.is_empty(),
        "diagnósticos inesperados: {:?}",
        result.diagnostics
    );

    // Índices de `lines` (solo el cuerpo entra en `lines`):
    // 0 = Label start, 1 = Goto end, 2 = Label end
    assert_eq!(result.program.resolve_label("start"), Some(0));
    assert_eq!(result.program.resolve_label("end"), Some(2));
    assert_eq!(result.program.resolve_label("no_existe"), None);

    // Programa vacío: sin etiquetas.
    assert_eq!(Program::new().resolve_label("cualquiera"), None);
    assert_eq!(Program::default().resolve_label("cualquiera"), None);
}

// ---------------------------------------------------------------------------
// AC3: Line::clone conserva expresión, argumentos y línea por variante
// ---------------------------------------------------------------------------

/// Evalúa `expr` con `health = value` y devuelve si el resultado es Int(1).
fn eval_expr(expr: &dyn Expr, value: i64) -> bool {
    let ctx = FakeContext::new().with("health", Value::Int(value));
    matches!(expr.eval(&ctx), Ok(Value::Int(1)))
}

/// Busca la primera línea que cumple `f` (el programa vive prestado).
fn find_line(program: &Program, f: impl Fn(&Line) -> bool) -> &Line {
    program
        .lines
        .iter()
        .find(|l| f(l))
        .expect("se esperaba una línea del programa en el test")
}

#[test]
fn test_line_clone_all_variants() {
    // `line` guarda el número de línea de origen, no el índice en `lines`.
    let text = "creature X\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\ngoto loop\nif health > 0 goto loop\nenergy = 5\nwait(1)\nloop:\n";
    let result = compile_text(text);
    assert!(
        result.diagnostics.is_empty(),
        "diagnósticos inesperados: {:?}",
        result.diagnostics
    );
    let program = result.program;

    // ---- Label { name, line }: el clon conserva nombre y línea de origen ----
    let start = find_line(
        &program,
        |l| matches!(l, Line::Label { name, .. } if name == "start"),
    );
    let (orig_name, orig_line) = match start {
        Line::Label { name, line } => (name.clone(), *line),
        _ => panic!("se esperaba Label"),
    };
    match start.clone() {
        Line::Label { name, line } => {
            assert_eq!(name, "start");
            assert_eq!(name, orig_name);
            assert_eq!(line, orig_line);
        }
        _ => panic!("se esperaba Label clonado"),
    }

    // ---- Goto { label, line } ----
    let goto = find_line(
        &program,
        |l| matches!(l, Line::Goto { label, .. } if label == "loop"),
    );
    let (orig_label, orig_line) = match goto {
        Line::Goto { label, line } => (label.clone(), *line),
        _ => panic!("se esperaba Goto"),
    };
    match goto.clone() {
        Line::Goto { label, line } => {
            assert_eq!(label, "loop");
            assert_eq!(label, orig_label);
            assert_eq!(line, orig_line);
        }
        _ => panic!("se esperaba Goto clonado"),
    }

    // ---- IfGoto { expr, label, line }: se conserva operador, etiqueta,
    // línea de origen y evaluación de la expresión ----
    let cond = find_line(&program, |l| matches!(l, Line::IfGoto { .. }));
    let (orig_label, orig_line) = match cond {
        Line::IfGoto { label, line, .. } => (label.clone(), *line),
        _ => panic!("se esperaba IfGoto"),
    };
    match cond {
        Line::IfGoto { expr, label, .. } => {
            assert_eq!(label, "loop");
            assert_eq!(expr.operator(), ">");
            assert!(eval_expr(&**expr, 5));
            assert!(!eval_expr(&**expr, 0));
        }
        _ => panic!("se esperaba IfGoto"),
    }
    match cond.clone() {
        Line::IfGoto { expr, label, line } => {
            assert_eq!(label, "loop");
            assert_eq!(label, orig_label);
            assert_eq!(line, orig_line);
            assert_eq!(expr.operator(), ">");
            assert!(eval_expr(&*expr, 5));
            assert!(!eval_expr(&*expr, 0));
        }
        _ => panic!("se esperaba IfGoto clonado"),
    }

    // ---- Assign { name, expr, line } ----
    let assign = find_line(&program, |l| matches!(l, Line::Assign { .. }));
    let (orig_name, orig_line) = match assign {
        Line::Assign { name, line, .. } => (name.clone(), *line),
        _ => panic!("se esperaba Assign"),
    };
    match assign {
        Line::Assign { name, expr, .. } => {
            assert_eq!(name, "energy");
            assert_eq!(expr.operator(), "");
            let ctx = FakeContext::new();
            assert_eq!(expr.eval(&ctx).unwrap(), Value::Int(5));
        }
        _ => panic!("se esperaba Assign"),
    }
    match assign.clone() {
        Line::Assign { name, expr, line } => {
            assert_eq!(name, "energy");
            assert_eq!(name, orig_name);
            assert_eq!(line, orig_line);
            let ctx = FakeContext::new();
            assert_eq!(expr.eval(&ctx).unwrap(), Value::Int(5));
        }
        _ => panic!("se esperaba Assign clonado"),
    }

    // ---- Action { action, args, line } ----
    let action_line = find_line(&program, |l| matches!(l, Line::Action { .. }));
    let orig_line = match action_line {
        Line::Action { line, .. } => *line,
        _ => panic!("se esperaba Action"),
    };
    match action_line {
        Line::Action { action, args, .. } => {
            assert_eq!(action.name(), "wait");
            assert_eq!(args.len(), 1);
            let ctx = FakeContext::new();
            assert_eq!(args[0].eval(&ctx).unwrap(), Value::Int(1));
        }
        _ => panic!("se esperaba Action"),
    }
    match action_line.clone() {
        Line::Action { action, args, line } => {
            assert_eq!(action.name(), "wait");
            assert_eq!(line, orig_line);
            assert_eq!(args.len(), 1);
            let ctx = FakeContext::new();
            assert_eq!(args[0].eval(&ctx).unwrap(), Value::Int(1));
        }
        _ => panic!("se esperaba Action clonado"),
    }
}
