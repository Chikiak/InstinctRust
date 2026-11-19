# INFORME_04_PROPUESTA_ISSUES — Issues candidatos para F5-ventana

Fase: F5-ventana. Rama: `develop`. Repositorio: `D:\Proyectos\InstinctRust`.
MASTER.md L78: F5 prevista con 9–13 issues. Aquí se proponen 11.
Regla: solo lectura (no editar código). Cada issue con ID, título, dependencias,
tamaño S/M/L, riesgo, y concepto de Rust que practica.

## Issues propuestos (11)

### F5-01 — Spike de librería gráfica offline
Elegir y verificar una librería gráfica que compile sin internet (egui/eframe
candidatas, pero NO presentes en el repo — INFORME_03 §1.5). Compilar un
programa mínimo que dibuje una rejilla y reciba un clic, usando solo
dependencias ya descargadas.
- Dependencias: ninguna (spike aislado).
- Tamaño: S. Riesgo: ALTO (si no compila offline, cambiar de librería o resolver descarga — MASTER.md L160).
- Concepto de Rust: selección de crate / FFI ausente; compilar sin red.

### F5-02 — Crear crate GUI en el workspace
Añadir `crates/instinct-gui` como tercer miembro de `Cargo.toml` raíz (L3-6),
con su `Cargo.toml` propio que depende de `instinct-core` y de la librería
gráfica elegida en F5-01. Añadir sus propias `[[test]]` independientes.
- Dependencias: F5-01.
- Tamaño: S. Riesgo: medio (cambia Cargo.toml raíz; patrón de 2 crates → 3).
- Concepto de Rust: workspace members, `[[test]]` por crate, `cargo test -p`.

### F5-03 — Adaptador de eventos para la GUI (WorldOutput)
Implementar un adaptador del puerto `WorldOutput` (`event/output.rs:10-13`) que
encole eventos en un buffer de registro legible por la GUI, conectado con
`sim.world.set_output(...)` (`world/ticks.rs:27`). Eventos: `Said`, `Died`,
`RuntimeError`, `Damaged`, `Roared` (`event/events.rs:13-57`).
- Dependencias: F5-02.
- Tamaño: M. Riesgo: bajo.
- Concepto de Rust: traits, `Box<dyn Trait>`, mutabilidad interior por puerto.

### F5-04 — Pantalla de carga y listado de catálogos
Leer recursos (terrenos, objetos, mapas, especies) y listarlos en la GUI;
mostrar diagnósticos de carga con archivo y línea sin abortar (precedente:
`cli/src/run.rs:16-21`, `cli/src/args.rs:49-124`).
- Dependencias: F5-02, F5-03.
- Tamaño: M. Riesgo: bajo.
- Concepto de Rust: `Result` / propagación de errores, `Vec<Diagnostic>`.

### F5-05 — Fase de preparación: colocar y quitar criaturas
UI para elegir mapa, elegir especie, colocar criatura (varias de la misma
especie) y quitar las colocadas por error. Validar bounds y celda libre sin
colocar sobre objeto ni criatura (precedente: `world/life.rs:20-42`,
`world/life.rs:80-87`).
- Dependencias: F5-04.
- Tamaño: M. Riesgo: medio (borrow checker en ciclo turno-mundo; mitigado por "intérprete lee, mundo escribe").
- Concepto de Rust: ownership y préstamos, `Result<_, Diagnostic>`.

### F5-06 — Controles de tiempo: play, pausa, paso y velocidades
Controles: play/pausa, avanzar un solo tick (`step_tick`, `sim/mod.rs:48`),
y al menos 3 velocidades distintas (ej. 2, 10, 30 ticks/seg). Durante la
simulación no se pueden añadir, quitar ni tocar criaturas (enunciado §4.1 paso 3).
- Dependencias: F5-05.
- Tamaño: M. Riesgo: medio (bucle de eventos + estado de simulación).
- Concepto de Rust: máquina de estados, bucle principal, `Duration`/tiempo.

### F5-07 — Render del snapshot de la ventana
Dibujar rejilla, criaturas, objetos y terrenos a partir de
`World::snapshot()` (`world/snapshot_view.rs:72`, DTO `WorldSnapshot` L14).
- Dependencias: F5-02, F5-04.
- Tamaño: M. Riesgo: bajo.
- Concepto de Rust: DTO inmutable, clonar datos para render (`snapshot_view.rs:73-144`).

### F5-08 — Visualización de rugidos (§2.4)
Mostrar `Event::Roared` (`event/events.rs:52-57`) y rugidos del tick anterior
(`world/roars.rs:38 previous_roars`) en el registro y destacar en la rejilla
la casilla del rugidor, tal como exige enunciado §2.4 (L142-143: "la
interfaz visual debe mostrarlo de alguna manera").
- Dependencias: F5-03, F5-07.
- Tamaño: S. Riesgo: bajo.
- Concepto de Rust: enums, pattern matching sobre `Event`.

### F5-09 — Botón de reinicio (volver a preparación)
Un botón devuelve a la fase de preparación (enunciado §4.1 paso 4, L388).
Dado que `Simulation` no tiene `Clone` (sim/mod.rs:18-20) y `World::clone`
reseta el RNG a seed 0 y quita `output` (`world/clone_debug.rs:24,28`), decidir
si se recrea el mundo desde `from_map` o se clona y se re-siembra.
- Dependencias: F5-05, F5-06.
- Tamaño: M. Riesgo: medio (semilla/determinismo al reiniciar).
- Concepto de Rust: clonar vs reconstruir, `set_rng_seed` (`ticks.rs:13`).

### F5-10 — Panel de registro (logs)
Panel con errores de carga al arranque, errores de ejecución (`RuntimeError`,
`event/events.rs:31-37`) y eventos (`Said`, `Died`, `Damaged`, `Roared`).
Precedente de streams: `cli/src/run.rs:13` (stdout/stderr).
- Dependencias: F5-03.
- Tamaño: S. Riesgo: bajo.
- Concepto de Rust: `Vec<Event>` / `drain_events` (`world/ticks.rs:54`).

### F5-11 — Banco de criaturas de prueba
Colección de criaturas y mapas, cada una probando una regla del enunciado, con
resultado esperado documentado (objetivo del proyecto: probar proyectos de
estudiantes, MASTER.md L78).
- Dependencias: F5-05, F5-07, F5-10.
- Tamaño: L. Riesgo: bajo.
- Concepto de Rust: pruebas de integración (`[[test]]` por área, patrón `tests/<area>/mod.rs`).

## Resumen de tallas y dependencias

- S: F5-01, F5-02, F5-08, F5-10 (4)
- M: F5-03, F5-04, F5-05, F5-06, F5-07, F5-09 (6)
- L: F5-11 (1)
- Total: 11 issues (dentro del rango 9–13 de MASTER.md L78).
- Grafo de dependencias: F5-01 → F5-02 → F5-03 → {F5-04, F5-10}; F5-04 → F5-05 → F5-06 → F5-09; F5-03,F5-07 → F5-08; F5-05,F5-07,F5-10 → F5-11.

Riesgo más alto: F5-01 (librería gráfica offline). Decisión abierta asociada:
ver INFORME_05.