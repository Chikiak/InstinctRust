# INFORME_02_FLUJO_GUI — Flujo §4.1 (4 pasos) vs CLI hoy

Fase: F5-ventana. Rama: `develop`. Repositorio: `D:\Proyectos\InstinctRust`.
Regla: solo lectura. Cada afirmación con `archivo:línea` + cita de código real.
Fuente: `docs/enunciado.md` §4.1 (L380-390) y §2.4 rugido (L141-143).

## 1. Requisito §4.1 (enunciado)

Paso 1 — Arranque (`enunciado.md:382`): "la aplicación lee los archivos .te de
terrains/, los .ob de objects/, los .map de maps/ y los .ins de creatures/, en
ese orden. Lo que carga bien queda disponible. Lo que no, se reporta con su
error y su línea, y la aplicación sigue funcionando."

Paso 2 — Preparación (`enunciado.md:384`): "El usuario escoge un mapa, y luego
coloca criaturas sobre él: elige una especie y señala una casilla. Debe poder
colocar varias criaturas de la misma especie y quitar las que coloque por
error. No se puede colocar sobre un objeto ni sobre otra criatura."

Paso 3 — Play (`enunciado.md:386`): "El usuario pulsa play. A partir de ahí la
única influencia posible sobre el mundo es la velocidad del tiempo: pausa,
avanzar un solo tick, y al menos tres velocidades distintas (por ejemplo 2, 10
y 30 ticks por segundo). No se pueden añadir, quitar ni tocar criaturas."

Paso 4 — Reinicio (`enunciado.md:388`): "También está la opción de reiniciar.
Un botón devuelve a la fase de preparación."

Registro (`enunciado.md:390`): "Debe haber un registro (logs) con los errores de
carga al arrancar, los errores de ejecución, y lo que sea que quieran mostrar."

Rugido visual (§2.4, `enunciado.md:142-143`): "roar(value) ... y la interfaz
visual debe mostrarlo de alguna manera."

## 2. Precedentes en la CLI (lo que ya existe)

### Flags y argumentos — `crates/instinct-cli/src/args.rs`

- `RunArgs { root, map_name, seed, ticks, every, places }` — `args.rs:4-11`.
- `--root` default `"examples"` — `args.rs:107`.
- `--map` requerido — `args.rs:114` ("missing required argument --map").
- `--seed u64` — `args.rs:69-72`.
- `--ticks i64 >= 0` — `args.rs:74-84` (rechaza < 0).
- `--every i64 >= 1` — `args.rs:86-96` (rechaza < 1).
- `--place Especie@x,y` (repetible) — `args.rs:21-44` (`parse_place`).
- Flag desconocida → error — `args.rs:101-103`.
- Validación de directorio raíz — `args.rs:110-112`.

### Orden de carga — `crates/instinct-cli/src/run.rs`

`execute_run` (L14):
1. `resources::load_resources(root)` — `run.rs:16`.
2. Diagnósticos de carga a stderr — `run.rs:19-21`.
3. Buscar mapa por nombre — `run.rs:24-41` (map not found → exit 1, L28-39).
4. Rechazar si no hay terrenos — `run.rs:43-46`.
5. Parsear `--place` (especies desconocidas abortan) — `run.rs:50-71`.
6. `World::from_map(map, &terrains, &objects)` — `run.rs:75`.
7. `place_creature` por cada placement — `run.rs:80-88`.
8. `Simulation::new(world)`, `set_output`, `set_rng_seed` — `run.rs:92-95`.

Orden coincide con §4.1 Paso 1: recursos → mapa → terrenos → colocaciones → mundo → criaturas.

### Streams — `crates/instinct-cli/src/run.rs` y `output.rs`

- C5 (run.rs:13): "The map and tick events go to stdout; diagnostics go to stderr".
- Mapa y eventos a stdout: `println!`/`print!` en `run.rs:101-108, 123-130`.
- Diagnósticos a stderr: `eprintln!` en `run.rs:19-21, 28-38, 44-45, 76-78, 83-86`.
- `CliOutput::emit` imprime eventos a stdout con `println!` — `output.rs:7-11` (L9).

### Códigos de salida — `crates/instinct-cli/src/main.rs`

- Sin args → stderr + exit 2 — `main.rs:18-21`.
- Error de parseo → exit 2 — `main.rs:26-29`.
- `exit(command.execute())` — `main.rs:32`; `execute_run` retorna 0 éxito / 1 error — `run.rs:40, 134`.

### Rugidos en CLI (§2.4)

- `Event::Roared` — `crates/instinct-core/src/event/events.rs:52-57` (tick, species, pos, text).
- `roars_current` / `roars_previous` — `crates/instinct-core/src/world/world.rs:57-60`.
- `World::add_roar` / `previous_roars` / `rotate_roars` — `roars.rs:26, 38, 45`.
- Percepciones F2: `roars_near` / `last_roar` (leen tick anterior, `tick - 1`).
- La GUI debe mostrar el rugido de alguna manera (§2.4, enunciado.md:142-143).

## 3. Evaluación por paso

| Paso | Veredicto | Evidencia |
|------|-----------|-----------|
| 1 Arranque | CUMPLE | Carga recursos, mapa, terrenos, colocaciones (`run.rs:16-20`); diagnósticos a stderr sin abortar (`run.rs:19-21`). |
| 2 Preparación | PARCIAL | `--map` y `--place` existen (`args.rs:114, 98-100`); `place_creature` valida bounds+celda libre (`life.rs:20-42`). FALTAN: UI para elegir mapa/especies, remover criatura colocada (`remove_creature` existe en núcleo `life.rs:80` pero no hay UI), validación visual de colisión. |
| 3 Play | PARCIAL | `--ticks`/`--every` existen (`args.rs:74-96`); `step_tick()` avanza 1 tick (`sim/mod.rs:48`); `set_rng_seed` determina (`sim/mod.rs:31`). FALTAN: pausa, paso individual a petición, ≥3 velocidades, prohibición de añadir/quitar criaturas durante la simulación. |
| 4 Reinicio | NO EXISTE | No hay método `reset`/`restart` en `World` ni `Simulation`; `World::clone` reseta RNG a 0 y quita `output` (`clone_debug.rs:24,28`). No hay botón/lógica para volver a preparación. |
| Registro | PARCIAL | CLI imprime eventos (`output.rs:9`) y diagnósticos (`run.rs:19-21`) pero no hay panel de registro persistente ni errores de ejecución capturados en UI. |
| Rugido §2.4 | PARCIAL (núcleo) | Núcleo emite `Event::Roared` y rota rugidos (`events.rs:52-57`, `roars.rs:45-47`), pero la CLI solo los imprime (`output.rs:9`); la GUI aún no los visualiza de forma destacada. |

## 4. Conclusión

El núcleo ya soporta todo el flujo §4.1 por API (`snapshot`, `WorldOutput`,
`step_tick`, `set_rng_seed`, `place_creature`/`remove_creature`, roars). Lo que
falta es exclusivamente la capa visual: ventana, controles de tiempo, panel de
registro y visualización de rugidos. Ningún cambio al núcleo es obligatorio
antes de construir la GUI; el núcleo es suficiente como backend de la ventana.