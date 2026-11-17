---
phase: R2
title: Refactor interno y preparación para la UI
status: todo
master: ../MASTER.md
fuentes:
  - ../../docs/PATRONES_DISENO_ARQUITECTURA.md
  - ../../docs/PROBLEMAS_Y_SOLUCIONES.md
issues: 7 desglosados en archivos propios (R2-01 a R2-07)
---

# R2 · Refactor interno y preparación para la UI

## Objetivo y entregable

Cerrar deuda interna antes de la ventana (F5): corregir errores lógicos remanentes,
eliminar malas prácticas (unwrap/expect silenciosos, strings en español en salida),
reducir duplicación DRY residual, respetar SOLID y DRY, y ampliar cobertura de pruebas.
También allanar el camino para F5 exponiendo una API de solo-lectura estructurada del
mundo que una GUI pueda consumir sin mutar el núcleo y sin cambios de API en el núcleo
salvo lo que un issue declare.

**Sin cambios de comportamiento observable:** misma semilla = misma historia, byte a byte.

Entregable observable: puertas verdes (`cargo test --workspace`, clippy `-D warnings`,
`fmt --check`), 0 comentarios en español en `crates/`, archivos de Rust ≤ ~300 líneas
salvo excepción anotada, y una API de snapshot de solo-lectura lista para que F5 la
use sin tocar el núcleo.

## Estado de partida

- F0–F4 y R1 completadas y fusionadas en `develop`. `cargo test --workspace`, clippy y
  fmt en verde. Determinismo `--seed 7` intacto.
- El módulo `actions/` ya está bien particionado (trait, catalog, context, helpers,
  combate, parseo) y sin WIP sin commitear.
- Árboles ya aplicados en F4: ECS (`Vec<Option<Creature>>` + `free_ids`), Flyweight
  (`Rc<Species>`), State (`CreatureState`), Visitor sobre AST, bus de eventos
  (`WorldOutput`), capas documentadas en `lib.rs`.
- Deuda remanente identificada (ver sección "Issues candidatas"):

  | Área | Problema | Dónde |
  |------|----------|-------|
  | Strings en español | `event/display.rs` aún usa "dice:", "daña", "ruge", "error en línea" | `event/display.rs` L16-46 |
  | API para UI | No hay snapshot estructurado de solo-lectura del mundo para renderizado | `world/draw.rs` devuelve ASCII `String` |
  | `unwrap_or(0)` silencioso | `CreatureSnapshot` devuelve 0 si la criatura no existe, mascarando bugs | `world/snapshot.rs` L17-71 |
  | `diagnostics` público | `World.diagnostics` es `pub` vec expuesto | `world/world.rs` L48 |
  | `clone_name` trivial | Función de 3 líneas que podría ser innecesaria | `resources/folder.rs` L153-155 |
  | Tests faltantes | `program_tests.rs` no existe; `Display` de eventos incompleto | `tests/` |

## Contratos compartidos

- **Cero cambios de comportamiento observable** salvo los que un issue declare y actualice
  en el mismo cambio. Puerta: `--seed 7` sobre los 4 ejemplos del anexo da historia
  idéntica.
- **API pública del núcleo estable:** F5 la consume sin cambios salvo declaración expresa
  del issue.
- **Sin `println!` en `instinct-core`; sin dependencias externas en el núcleo; sin
  `f32`/`f64`.**
- **Techo ~300 líneas por archivo** salvo excepción justificada y anotada en el issue.
- **Cada issue que corrige un bug lleva test que falle primero (TDD).**
- `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo fmt --all --check` en verde tras cada issue; `git status` limpio tras commit.

## Decisiones de la fase

1. **R2-01 va primero:** corrige strings en español que varios asserts de tests pueden
   afirmar; hacerlo antes de tocar otros archivos evita solapes.
2. **No se reabren patrones ya aplicados en F4** (ECS, Flyweight, State, Visitor, bus,
   capas) salvo que un issue demuestre un bug real con prueba.
3. **Todo lo que sea E/S o impresión queda fuera del núcleo.** La UI de F5 será un
   adaptador más mediante el puerto `WorldOutput`.
4. **El snapshot para UI es de solo-lectura:** expone clonaciones de datos, no referencias
   mutables al mundo.

## Issues

| ID | Título | Problema / patrón que cubre | Depende de | Tamaño | Estado | Archivo de issue |
|----|--------|----------------------------|------------|--------|--------|------------------|
| R2-01 | Strings de eventos al inglés | `dice:`/`daña`/`ruge`/`error en línea` → inglés | — | S | done | r2-01-event-strings-english.md |
| R2-02 | Snapshot estructurado de solo-lectura para renderizado | API pre-UI: estado del mundo sin mutar | R2-01 | M | done | r2-02-world-snapshot-readonly.md |
| R2-03 | Auditar errores lógicos remanentes | `unwrap_or(0)`, rango de `pc`, casts signo→`usize`, `checked_*` | R2-01 | M | todo | r2-03-logic-errors-audit.md |
| R2-04 | DRY residual y cohesión en `actions/` | Revisar attack/consume/combat/shared/parse | R2-01 | M | todo | r2-04-dry-actions.md |
| R2-05 | CLI: separación de streams y validaciones | Revisar stdout/stderr, `--ticks`, `--every`, `--root` | R2-01 | S | todo | r2-05-cli-streams-validation.md |
| R2-06 | Testing coverage gaps | `program_tests.rs`, `Display` de eventos, edge cases | R2-02, R2-03 | M | todo | r2-06-testing-coverage.md |
| R2-07 | Higiene final y verificación de salida | Archivos >300 L, comentarios ES, puertas verdes | R2-02…R2-06 | S | todo | r2-07-higiene-final.md |

### Orden y dependencias

- R2-01 es raíz: toca strings que tests afirman; hacerlo primero evita solapes.
- R2-02 depende de R2-01 (strings de eventos en el snapshot).
- R2-03 depende de R2-01 (mensajes de diagnóstico en inglés).
- R2-04 depende de R2-01 (helpers compartidos pueden usar strings en inglés).
- R2-05 depende de R2-01 (mensajes de CLI en inglés).
- R2-06 depende de R2-02 y R2-03 (tests del snapshot y de errores corregidos).
- R2-07 cierra: verifica que todo está verde y limpio.

## Cobertura

| Elemento | Dónde se cubre |
|----------|----------------|
| Strings en inglés en salida de eventos | R2-01 |
| API de snapshot de solo-lectura para GUI | R2-02 |
| Errores lógicos (`unwrap_or`, `pc`, casts, `checked_*`) | R2-03 |
| DRY en `actions/` | R2-04 |
| Separación stdout/stderr en CLI | R2-05 |
| `program_tests.rs`, `Display` de eventos, edge cases | R2-06 |
| Techo 300 líneas, 0 comentarios ES, puertas verdes | R2-07 |

## Verificación de salida de la fase

- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- Dos corridas con `--seed 7` sobre los 4 ejemplos del anexo dan la misma historia byte a
  byte que antes de la fase.
- `git grep` de comentarios en español sobre `crates/` → 0 coincidencias.
- Ningún archivo `.rs` de `crates/` por encima de 300 líneas salvo excepción anotada.
- `git status --short` limpio tras cada commit.

## Prompt de sesión

```text
Implementa el issue plan/R2-refactor-pre-ui/<ID>-<slug>.md.
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
Esta fase es refactor sin cambio de comportamiento: la puerta de salida incluye
el determinismo con semilla fija.
Ejecuta los comandos de "Verificación" y no lo des por terminado hasta cubrir cada
criterio de aceptación con evidencia. Al acabar: actualiza `status` y rellena
"Notas de implementación". Si el issue contradice el repo o te bloquea, para y
repórtalo en lugar de improvisar.
```

## Registro de ejecución

| Issue | Patrón / enfoque | Motivación | Coste asumido | Estado | Medición |
|-------|------------------|------------|---------------|--------|----------|
| R2-01 | Rename de strings | Salida de eventos en español | Tests de Display afectados | done | 4 strings renombradas / 8 asserts actualizados / puertas verdes |
| R2-02 | Snapshot / DTO de solo-lectura | F5 necesita consumir el mundo sin mutar | Nuevo módulo o archivo | done | `cargo test --workspace` exit 0 / clippy `-D warnings` exit 0 / `fmt --check` exit 0 / determinismo `--seed 7` dos corridas idénticas (SHA256 1401F3F902C2AD9E0F09247E7D5E21A534C968C52A8B9589AB2FB611BCBC172C) |
| R2-03 | Corrección de implementación | `unwrap_or(0)` silencioso, casts inseguros | Ninguno: solo pruebas + código | todo | — |
| R2-04 | DRY / extracción de helpers | Duplicación residual en `actions/` | Ninguno: solo movimiento de código | todo | — |
| R2-05 | Revisión de Contracts CLI | Streams mezclados, validaciones | Ninguno | todo | — |
| R2-06 | Pruebas | Cobertura de huecos detectados | Más tests | todo | — |
| R2-07 | Higiene | Techo de líneas, comentarios ES | Ninguno | todo | — |

<!-- Se rellena al avanzar: enfoque, motivación, coste, mediciones. -->