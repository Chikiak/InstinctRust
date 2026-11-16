---
id: R2-07
title: Higiene final y verificación de salida
status: todo
phase: R2
depends_on: [R2-02, R2-03, R2-04, R2-05, R2-06]
size: S
file: r2-07-higiene-final.md
---

# R2-07 · Higiene final y verificación de salida

## Leer primero

Este issue cierra la fase: verifica que todos los archivos de Rust en `crates/` estén
por debajo de ~300 líneas salvo excepción anotada, que no haya comentarios en español,
y que todas las puertas de salida estén verdes.

## Qué verificar

1. **Tamaño de archivos:** ningún archivo `.rs` bajo `crates/` supera las ~300 líneas
   salvo excepción anotada y justificada (como `expr/parse.rs` para descenso recursivo
   o `tests/actions/reproduce.rs` para tests granulares).
2. **Comentarios en español:** `git grep` de comentarios en español sobre `crates/` →
   0 coincidencias.
3. **Strings en español en salida:** `git grep` de `"dice:"`, `"daña"`, `"ruge"`,
   `"error en línea"` sobre `crates/` → 0 coincidencias.
4. **Puertas verdes:** `cargo test --workspace`, `cargo clippy --workspace
   --all-targets -- -D warnings`, `cargo fmt --all --check` todos en exit 0.
5. **Determinismo:** `--seed 7` sobre los 4 ejemplos del anexo da historia idéntica.
6. **Árbol limpio:** `git status --short` vacío tras commit.

## Criterios de aceptación

1. Todas las verificaciones arriba pasan.
2. `git status --short` limpio.
3. El ledger de la fase (`.opencode/ledgers/r2-refactor-pre-ui.md`) está actualizado
   con todos los issues como `done` y sus merge hashes.
4. `PHASE.md` tiene el registro de ejecución completado.

## Notas de implementación

<!-- Rellenar al completar: excepciones de tamaño anotadas, resultados de las puertas. -->
