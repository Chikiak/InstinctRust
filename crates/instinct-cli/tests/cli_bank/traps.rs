//! Diagnosticos de trampas y carga tolerable desde la terminal (AC6, AC7, AC8).

use super::{run_cli, traps_dir};

/// Todas las trampas de compilacion (una por clase de error del enunciado 2.8).
const TRAPS: &[&str] = &[
    "bad_line",
    "unknown_label",
    "dup_label",
    "missing_creature",
    "missing_start",
    "bad_expr",
    "assign_perception",
    "assign_constant",
    "unknown_action",
    "unknown_function",
    "wrong_arity",
];

/// Un diagnostico tiene forma `archivo:linea: mensaje`, es decir, una linea
/// que contiene un patron `:<digitos>:` (coincide con `.*:\d+:`).
fn diagnostico_con_linea(stderr: &str) -> bool {
    stderr.lines().any(|line| {
        let line_bytes = line.as_bytes();
        let mut i = 0;
        while i < line_bytes.len() {
            if line_bytes[i] == b':' {
                let mut j = i + 1;
                while j < line_bytes.len() && line_bytes[j].is_ascii_digit() {
                    j += 1;
                }
                if j > i + 1 && j < line_bytes.len() && line_bytes[j] == b':' {
                    return true;
                }
            }
            i += 1;
        }
        false
    })
}

// AC6: `check` rechaza cada trampa con exit 1 y un diagnostico con archivo y
// linea en stderr.
#[test]
fn check_rejects_every_trap_with_line() {
    for trampa in TRAPS {
        let path = format!("{}/creatures/{}.ins", traps_dir(), trampa);
        let (code, _stdout, stderr) = run_cli(&["check", &path]);
        assert_eq!(code, 1, "{trampa}: check debe salir 1, stderr: {stderr}");
        assert!(
            !stderr.is_empty(),
            "{trampa}: stderr no vacio, got: {stderr}"
        );
        assert!(
            diagnostico_con_linea(&stderr),
            "{trampa}: stderr debe tener una linea 'archivo:linea: mensaje', got: {stderr}"
        );
    }
}

// AC7: `check` acepta la criatura valida con OK y exit 0.
#[test]
fn check_accepts_valid_creature() {
    let path = format!("{}/creatures/valid.ins", traps_dir());
    let (code, stdout, stderr) = run_cli(&["check", &path]);
    assert_eq!(code, 0, "valid.ins debe salir 0, stderr: {stderr}");
    assert_eq!(
        stdout.trim(),
        "OK",
        "valid.ins debe imprimir OK, got: {stdout}"
    );
    assert!(stderr.is_empty(), "valid.ins stderr vacio, got: {stderr}");
}

// AC8: cargar examples/traps/ con run sigue funcionando: la criatura valida
// esta disponible, se coloca y simula, y los diagnosticos de las trampas salen
// por stderr con exit 0.
#[test]
fn load_traps_root_keeps_valid_creature() {
    let (code, stdout, stderr) = run_cli(&[
        "run",
        "--root",
        &traps_dir(),
        "--map",
        "Arena",
        "--seed",
        "1",
        "--ticks",
        "2",
        "--every",
        "1",
        "--place",
        "Valid@2,2",
    ]);
    assert_eq!(code, 0, "run sobre traps debe salir 0, stderr: {stderr}");
    // La valida simula: 'V' en (2,2) en el tick 0.
    assert!(
        stdout.contains("..V......"),
        "tick 0: Valid en (2,2), got:\n{stdout}"
    );
    // Los diagnosticos de las trampas salen por stderr (carga tolerante).
    assert!(
        !stderr.is_empty(),
        "stderr debe tener diagnosticos de las trampas, got: {stderr}"
    );
    assert!(
        stderr.contains("bad_line") || stderr.contains("unknown_label"),
        "stderr debe nombrar al menos una trampa, got: {stderr}"
    );
}
