# INFORME_03_DESACOPLAMIENTO — Reglas de dependencia y límites del GUI

Fase: F5-ventana. Rama: `develop`. Repositorio: `D:\Proyectos\InstinctRust`.
Regla: solo lectura. Cada afirmación con `archivo:línea` + cita de código real.

## 1. Reglas de dependencia del repositorio

### Núcleo sin dependencias externas ni E/S

`crates/instinct-core/Cargo.toml:74`: la sección `[dependencies]` está vacía
(línea 74, fin del archivo; 17 secciones `[[test]]` en L6-L73, 0 dependencias).
Cita: `[dependencies]` (L74) seguido inmediatamente de fin de archivo.

Grep de `println!` en `crates/instinct-core/src`: 0 matches. El núcleo no hace
E/S de consola.

### CLI depende del núcleo, no al revés

`crates/instinct-cli/Cargo.toml:26`:
```
instinct-core = { path = "../instinct-core" }
```
Dependencia única del CLI. El núcleo no conoce al CLI.

### Puerto de eventos: única superficie de salida del núcleo

`WorldOutput` trait — `crates/instinct-core/src/event/output.rs:10-13`:
```rust
pub trait WorldOutput {
    fn emit(&mut self, event: &Event); // L13
}
```
Reenvío en `World::emit_event` — `crates/instinct-core/src/world/ticks.rs:44-51`.
Único implementador hoy: `CliOutput` — `crates/instinct-cli/src/output.rs:7-11`
(L9: `println!("{event}");`).

### Workspace

`Cargo.toml` raíz (L3-6):
```
members = [
    "crates/instinct-core",
    "crates/instinct-cli",
]
```
Solo 2 miembros. No hay carpeta `tests/` en raíz ni `program_tests.rs`.

### Biblioteca gráfica: NO presente (offline)

Grep de `egui`, `eframe`, `macroquad`, `bevy`, `iced`, `tao`, `winit` en todos
los `*.toml`: 0 matches. `Cargo.lock` sin matches de crate gráfico.
Conclusión: **egui/eframe = NO presente** hoy; añadir una requiere descarga, y
el proyecto asume "sin internet durante las sesiones" (MASTER.md L23, L48).
Primero hay que descargar la librería en una conexión disponible (atajo 4,
MASTER.md L48) o elegir una librería ya en caché.

## 2. Qué puede y qué NO puede hacer el crate GUI

### GUI PUEDE (sobre la instancia propia del mundo/simulación)

- Leer el snapshot: `World::snapshot()` (`snapshot_view.rs:72`, `&self`, no muta).
- Consumir eventos vía `WorldOutput`: implementar `emit(&mut self, event: &Event)`
  (`output.rs:10-13`) y conectar con `sim.world.set_output(...)` (`ticks.rs:27`).
- Avanzar la simulación: `Simulation::step_tick() -> Vec<Event>` (`sim/mod.rs:48`).
- Consultar tick: `Simulation::tick()` / `World::tick()` (`sim/mod.rs:36`, `ticks.rs:32`).
- Reseed determinista: `set_rng_seed(u64)` (`ticks.rs:13`, `sim/mod.rs:31`).
- Leer catálogos (campos `pub`): `terrains`, `objects`, `species`, `catalog`
  (`world.rs:25, 27, 33, 50`).
- Colocar / quitar criaturas: `place_creature` / `remove_creature` (`life.rs:20, 80`),
  devuelven `Result` (nunca abortan).
- Leer rugidos previos: `previous_roars()` (`roars.rs:38`) y eventos `Roared`
  (`events.rs:52-57`).
- Clonar el mundo para inspección: `World::clone` (`clone_debug.rs:7`, reseta RNG a 0).

### GUI NO PUEDE (sin modificar el núcleo)

- Hacer `println!` / E/S dentro de `crates/instinct-core` (0 matches; regla de arquitectura).
- Añadir dependencias al núcleo (`Cargo.toml` del núcleo vacío en `[dependencies]`, L74).
- Mutar `World` directamente salvo por su propia instancia (el núcleo es dueño de sus catálogos; `world.rs:24-27` aclara que los reserves se mutan en los catálogos del mundo).
- Romper el determinismo: la secuencia de `random` y el orden de turnos salen del mismo generador semillado dentro del mundo (`sim/mod.rs:5-6`, `enunciado.md:231`).
- Acceder a `Simulation` sin su `World` público: `Simulation.world` es `pub` (sim/mod.rs:19) pero no hay `Simulation::clone` (sim/mod.rs:18-20).

### Implicación de diseño (puertos y adaptadores)

El núcleo no sabe quién imprime los eventos (`output.rs:1-4` docstring). La GUI
es un adaptador más del puerto `WorldOutput`, al igual que `CliOutput` hoy.
Esto cumple §5 del enunciado: "Al simulador le da igual si hay o no una
interfaz. Si no pueden correr una simulación completa desde la terminal, sin
abrir visual, está mal separado." (`enunciado.md:411`).

## 3. Impacto de añadir un 4º crate al workspace

Hoy: `instinct-core`, `instinct-cli` (Cargo.toml raíz L3-6).

Para añadir `instinct-gui`:
1. Agregar `"crates/instinct-gui"` a `members` en `Cargo.toml` raíz (L3-6).
2. Crear `crates/instinct-gui/Cargo.toml` con dependencia `instinct-core = { path = "../instinct-core" }` y la librería gráfica elegida (solo en GUI, nunca en núcleo).
3. Añadir sus propias secciones `[[test]]` independientes por crate (patrón actual: `tests/<area>/mod.rs` en cada crate; core tiene 17, cli tiene 4).
4. `cargo test -p <name>` aísla cada crate; `cargo test --workspace` incluye todo y es el criterio de salida (MASTER.md L110).

Riesgos a registrar como decisiones abiertas (ver INFORME_05): elección de
librería gráfica offline, ubicación del crate (`crates/instinct-gui`), y
tiempo de compilación del workspace al añadir una librería gráfica con muchas
dependencias transitivas.