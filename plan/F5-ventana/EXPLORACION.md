# Brief de exploración — F5-ventana (parte visual)

Fase a planificar: **F5-ventana**. Objetivo del usuario: crear y conectar la parte visual,
desacoplada y con buenos principios. NO se escribe código en esta exploración: solo informes.

## Contexto que el coordinador debe leer primero

- `plan/MASTER.md` (fase F5 = Ventana + Banco de criaturas; reglas de replanificación)
- `plan/R2-refactor-pre-ui/PHASE.md` (estado de partida: API pre-UI lista)
- `docs/enunciado.md` (secciones 4 y 4.1: flujo de la aplicación; 2.4: roar debe mostrarse)
- `docs/PATRONES_DISENO_ARQUITECTURA.md` (puertos/adaptadores y capas)
- `crates/instinct-core/src/lib.rs`, `world/snapshot_view.rs`, `world/world.rs`,
  `sim/mod.rs`, `event/output.rs`, `event/events.rs`, `resources/mod.rs`
- `crates/instinct-cli/src/*` (`main.rs`, `args.rs`, `run.rs`, `check.rs`, `output.rs`, `command.rs`)

## Fan-out obligatorio (explorers en paralelo)

Despacha explorers separados y en paralelo, uno por área, y consolida sus hallazgos:

1. **API del núcleo para la GUI** (`world/snapshot_view.rs`, `world/world.rs`, `sim/mod.rs`):
   inventario exacto de lo público hoy; huecos para la UI (quitar criatura colocada,
   reiniciar, validación de colisión al colocar, listado de mapas/especies, velocidad,
   clonado de mundo para el reinicio); estabilidad de la API (¿qué puede cambiar sin
   romper CLI/tests?).
2. **Flujo de la GUI según el enunciado 4.1**: estados (carga → preparación → corriendo →
   pausa/reinicio), controles mínimos (play, pausa, paso, ≥3 velocidades, reiniciar),
   panel de registro (errores de carga, errores de ejecución, eventos), visualización de
   roars (2.4). Precedentes en la CLI (flags, orden de carga, diagnósticos).
3. **Desacoplamiento y límites**: qué puede y qué no puede hacer el crate GUI; reglas de
   dependencia (núcleo sin E/S, GUI como adaptador del puerto `WorldOutput`); criterios
   para elegir biblioteca gráfica en Windows sin internet la mayor parte del día
   (egui/eframe como candidata principal: pura Rust, sin libs de sistema; el spike debe
   compilar con las dependencias ya descargadas en el primer issue).
4. **Pruebas y estructura del repo**: cómo se organizan los tests de integración
   (`tests/<area>/mod.rs` + `Cargo.toml [[test]]`), convenciones (inglés en código y
   asserts), techo ~300 líneas por archivo, y qué cambia al añadir un cuarto crate
   (workspace members, compartición de `[[test]]`, tiempo de compilación).

## Salidas obligatorias (el implementador escribe estos archivos)

- `plan/F5-ventana/INFORME_01_API_NUCLEO.md`
- `plan/F5-ventana/INFORME_02_FLUJO_GUI.md`
- `plan/F5-ventana/INFORME_03_DESACOPLAMIENTO.md`
- `plan/F5-ventana/INFORME_04_PROPUESTA_ISSUES.md` (issues candidatos con ID, título,
  dependencias, tamaño S/M/L, riesgo, y qué concepto de Rust practica cada uno)
- `plan/F5-ventana/INFORME_05_DECISIONES_ABIERTAS.md` (todo lo que requiera decisión
  humana; el coordinador NO pregunta: registra cada una con opciones y recomendación)

## Reglas duras

- Cero cambios en `crates/`, `tests/`, `examples/` y `Cargo.toml`. Solo se escriben los
  INFORME_*md anteriores.
- Cero preguntas al usuario: lo que bloquee se registra como `DECISION_ABIERTA`.
- Los explorers son de solo lectura. El implementador solo escribe los informes.
- Verifica cada afirmación contra el código real (archivo y línea); sin suposiciones.
- Al terminar, el coordinador-issue devuelve un resumen de ≤15 líneas con la ruta de cada
  informe y el recuento de issues propuestos.