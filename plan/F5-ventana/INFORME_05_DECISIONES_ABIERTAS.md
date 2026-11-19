# INFORME_05_DECISIONES_ABIERTAS — Decisiones que requieren choix humano

Fase: F5-ventana. Rama: `develop`. Repositorio: `D:\Proyectos\InstinctRust`.
Cada decisión registra opciones y recomendación. CERO preguntas al usuario.
Lo que bloquee pasa aquí como `DECISION_ABIERTA`.

## DECISION_ABIERTA_1 — Biblioteca gráfica para la GUI (ventana)

Contexto: egui/eframe NO están presentes en el repo (0 matches en *.toml y
Cargo.lock — INFORME_03 §1.5). El proyecto asume "sin internet durante las
sesiones" (MASTER.md L23, L48), por lo que añadir una librería gráfica requiere
habérsela descargado previamente (atajo 4, MASTER.md L48) o usar una ya en
caché. El primer issue de F5 es un spike que compila sin conexión (MASTER.md L140).

Opciones:
- egui + eframe: pura Rust, sin libs de sistema, candidata principal del brief (EXPLORACION.md L32).
- macroquad: simple, una sola dependencia, buen soporte Windows; menos convencional para herramientas.
- iced / bevy / tao+winit: más pesadas, más dependencias transitivas, mayor riesgo offline.

Recomendación: egui + eframe (candidata principal, pura Rust, sin libs de sistema). Validar en el spike F5-01 que compila sin red con dependencias ya descargadas.

Dueño sugerido: F5 (quien ejecute F5-01).

## DECISION_ABIERTA_2 — Ubicación y nombre del crate GUI

Contexto: workspace tiene 2 miembros (Cargo.toml raíz L3-6). Añadir un 4º crate
requiere editar `members` (Cargo.toml raíz L3-6) y crear su `Cargo.toml` con
`[[test]]` propias (patrón `tests/<area>/mod.rs`, INFORME_04).

Opciones:
- `crates/instinct-gui` (coherente con `instinct-core`, `instinct-cli`).
- `crates/instinct-window`.

Recomendación: `crates/instinct-gui`, nombre paralelo a los existentes.

## DECISION_ABIERTA_3 — Mecanismo de reinicio (volver a preparación)

Contexto: `Simulation` no tiene `Clone` (sim/mod.rs:18-20). `World::clone`
existe pero reseta el RNG a seed 0 y deja `output: None`
(`world/clone_debug.rs:24,28`). El reinicio (§4.1 paso 4, enunciado.md:388)
debe devolver a la fase de preparación preservando la semilla para reproducibilidad.

Opciones:
- Recrear `World` desde `from_map` + `place_creature` y `set_rng_seed(seed)` (semilla explícita, determinista).
- Clonar `World` original capturado al inicio de la preparación y `set_rng_seed(seed)` sobre el clon.

Recomendación: recrear desde `from_map` + `place_creature` + `set_rng_seed(seed)`, porque `World::clone` reseta RNG a 0 (`clone_debug.rs:24`) y perdería la semilla del usuario.

## DECISION_ABIERTA_4 — Velocidades concretas (ticks por segundo)

Contexto: §4.1 exige "al menos tres velocidades distintas (por ejemplo 2, 10 y 30 ticks por segundo)" (enunciado.md:386). El núcleo no tiene concepto de velocidad; `step_tick` avanza 1 tick (sim/mod.rs:48). La velocidad es solo del bucle de la GUI.

Opciones: fijar 2/10/30 (ejemplo del enunciado); permitir personalizar; exponer un slider.

Recomendación: usar exactamente 2 / 10 / 30 ticks por segundo (cumple el enunciado sin inventar valores).

## DECISION_ABIERTA_5 — ¿Puede el crate GUI tener dependencias mientras el núcleo tiene 0?

Contexto: núcleo con `[dependencies]` vacío (`crates/instinct-core/Cargo.toml:74`) y 0 `println!`. La regla de arquitectura es que el núcleo no haga E/S ni tenga deps externas; el GUI es un adaptador del puerto `WorldOutput` (INFORME_03 §1.3).

Opciones:
- Permitir dependencias gráficas solo en `instinct-gui` (núcleo intacto).
- Intentar mantener también el GUI sin deps externas (imposible para una ventana real sin bindings de sistema).

Recomendación: permitir dependencias gráficas solo en `instinct-gui`; el núcleo queda con 0 deps y 0 E/S.

## DECISION_ABIERTA_6 — Formato y ubicación del banco de criaturas de prueba (F5-11)

Contexto: F5-11 propone una colección de criaturas/mapas con resultado esperado
documentado (MASTER.md L78). No hay formato definido ni carpeta prevista.

Opciones:
- Carpeta `examples/` con subcarpetas por regla y un archivo `.md` de resultado esperado por cada una.
- Formato JSON/YAML nuevo dentro del crate GUI.

Recomendación: reutilizar `examples/` (ya existe como raíz por defecto `--root`, `cli/src/args.rs:107`) con un archivo de expectativa por criatura; no inventar nuevo formato.

## Riesgos bloqueantes pendientes

- F5-01 no compila sin conexión → cambiar de librería o resolver descarga antes de seguir (MASTER.md L160).
- Si un issue toca el mismo archivo que otro → usar el grafo de dependencias explícito (INFORME_04 resumen).
- Si F5 supera 8 issues → dividir la fase (MASTER.md L156); aquí son 11, dentro de 9–13 pero cerca del techo, considerar dividir si crece.