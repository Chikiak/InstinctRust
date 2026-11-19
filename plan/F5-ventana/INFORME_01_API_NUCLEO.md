# INFORME_01_API_NUCLEO — Inventario de API pública del núcleo para la GUI

Fase: F5-ventana. Rama: `develop`. Repositorio: `D:\Proyectos\InstinctRust`.
Regla: solo lectura (no editar crates/, tests/, examples/, Cargo.toml).
Cada símbolo citado con `archivo:línea` + firma + cita de código real.
Código y asserts en inglés. Techo ~300 líneas.

## 1. DTOs de snapshot (lectura única, seguros para la GUI)

Todos en `crates/instinct-core/src/world/snapshot_view.rs`, derivados
`Clone, Debug, PartialEq, Eq` (L13-L27, L35, L50, L58).

**`WorldSnapshot`** — `snapshot_view.rs:14`
```rust
pub struct WorldSnapshot {
    pub columns: usize,      // L15
    pub rows: usize,         // L16
    pub ticks: u64,          // L17
    pub map_name: String,    // L18
    pub cells: Vec<Vec<CellSnapshot>>,   // L19
    pub creatures: Vec<LiveCreature>,    // L20
    pub objects: Vec<ObjectInfo>,        // L21
    pub terrains: Vec<TerrainInfo>,      // L22
    pub events: Vec<Event>,              // L23
}
```
Cita: `pub struct WorldSnapshot {` (L14). Construido por `World::snapshot()` (L72).

**`CellSnapshot`** — `snapshot_view.rs:28`
```rust
pub struct CellSnapshot {
    pub terrain_char: char,        // L29
    pub object_char: Option<char>, // L30
    pub creature: Option<CreatureId>, // L31
}
```

**`LiveCreature`** — `snapshot_view.rs:36`
```rust
pub struct LiveCreature {
    pub id: CreatureId,       // L37
    pub species_name: String, // L38
    pub faction: String,      // L39
    pub x: usize,             // L40
    pub y: usize,             // L41
    pub health: i64,          // L42
    pub age: i64,             // L43
    pub lifespan: i64,        // L44
    pub state: CreatureState, // L45
    pub pc: usize,            // L46
}
```

**`ObjectInfo`** — `snapshot_view.rs:51`; **`TerrainInfo`** — `snapshot_view.rs:59`
```rust
pub struct ObjectInfo { pub index: usize, pub name: String, pub char: char, pub reserve: i64 } // L51-L56
pub struct TerrainInfo { pub name: String, pub char: char, pub reserve: i64, pub resource_max: i64 } // L59-L65
```

**`World::snapshot()`** — `snapshot_view.rs:72`
Firma: `pub fn snapshot(&self) -> WorldSnapshot` (L72). Solo lectura (`&self`), no muta.
Cita clave: `pub fn snapshot(&self) -> WorldSnapshot {` (L72) y construye el DTO clonando datos
(L73-L144), incluyendo `events: self.events.clone()` (L143).

## 2. API pública de `World` para la GUI

### Construcción y consulta de estado

**`World::from_map()`** — `crates/instinct-core/src/world/world.rs:71`
Firma: `pub fn from_map(map: &Map, terrains: &[Terrain], objects: &[Object]) -> Self` (L71).
Cita: `pub fn from_map(map: &Map, terrains: &[Terrain], objects: &[Object]) -> Self {` (L71).
Retorna `World` con `ticks: 0` (L152) y catálogos vacíos para especies/criaturas (L149-L151).

### Ciclo de criaturas

**`World::place_creature()`** — `crates/instinct-core/src/world/life.rs:20`
Firma: `pub fn place_creature(&mut self, species: &Rc<Species>, x: usize, y: usize) -> Result<CreatureId, Diagnostic>` (L20-25).
Valida bounds (L26-35) y `cell_is_free` (L36-42). World unchanged on failure.

**`World::remove_creature()`** — `crates/instinct-core/src/world/life.rs:80`
Firma: `pub fn remove_creature(&mut self, creature_id: CreatureId) -> Result<(), Diagnostic>` (L80).
Libera la celda y el slot (L82-85).

**`World::live_creature_ids()`** — `crates/instinct-core/src/world/life.rs:91`
Firma: `pub fn live_creature_ids(&self) -> Vec<CreatureId>` (L91). IDs ascendentes, O(1) por slot.

**`World::creature_position()`** — `crates/instinct-core/src/world/life.rs:101`
Firma: `pub fn creature_position(&self, creature_id: CreatureId) -> Result<Pos, Diagnostic>` (L101).

### Ticks, RNG y eventos

**`World::tick()`** — `crates/instinct-core/src/world/ticks.rs:32`
Firma: `pub fn tick(&self) -> u64` (L32). Cita: `pub fn tick(&self) -> u64 {` (L32).

**`World::advance_tick()`** — `crates/instinct-core/src/world/ticks.rs:37`
Firma: `pub fn advance_tick(&mut self)` (L37).

**`World::set_rng_seed()`** — `crates/instinct-core/src/world/ticks.rs:13`
Firma: `pub fn set_rng_seed(&mut self, seed: u64)` (L13). Determinismo: misma semilla = misma historia.

**`World::set_output()`** — `crates/instinct-core/src/world/ticks.rs:27`
Firma: `pub fn set_output(&mut self, output: Box<dyn WorldOutput>)` (L27). Conecta puerto de eventos.

**`World::emit_event()`** — `crates/instinct-core/src/world/ticks.rs:44`
Firma: `pub fn emit_event(&mut self, event: Event)` (L44). Reenvía al puerto y guarda en log (L47-50).

**`World::drain_events()`** — `crates/instinct-core/src/world/ticks.rs:54`
Firma: `pub fn drain_events(&mut self) -> Vec<Event>` (L54). Cita: `std::mem::take(&mut self.events)` (L55).

**`World::set_rng_source()`** — `crates/instinct-core/src/world/ticks.rs:19`
Firma: `pub fn set_rng_source(&mut self, rng: Box<dyn RngSource>)` (L19). Para `TestRng` en tests.

### Rugidos (F2 / §2.4)

**`World::add_roar()`** — `crates/instinct-core/src/world/roars.rs:26`
Firma: `pub fn add_roar(&mut self, creature_id: CreatureId, species: String, pos: Pos, text: String)` (L26).

**`World::previous_roars()`** — `crates/instinct-core/src/world/roars.rs:38`
Firma: `pub fn previous_roars(&self) -> &[Roar]` (L38). Rugidos del tick anterior.

**`World::rotate_roars()`** — `crates/instinct-core/src/world/roars.rs:45`
Firma: `pub fn rotate_roars(&mut self)` (L45). Llama `sim.rs` al final de cada tick.

### Simulación

**`Simulation::new()`** — `crates/instinct-core/src/sim/mod.rs:25`
Firma: `pub fn new(world: World) -> Self` (L25). Campo público `pub world: World` (L19).

**`Simulation::set_rng_seed()`** — `crates/instinct-core/src/sim/mod.rs:31`
Firma: `pub fn set_rng_seed(&mut self, seed: u64)` (L31).

**`Simulation::tick()`** — `crates/instinct-core/src/sim/mod.rs:36`
Firma: `pub fn tick(&self) -> u64` (L36).

**`Simulation::world()`** — `crates/instinct-core/src/sim/mod.rs:41`
Firma: `pub fn world(&self) -> &World` (L41).

**`Simulation::step_tick()`** — `crates/instinct-core/src/sim/mod.rs:48`
Firma: `pub fn step_tick(&mut self) -> Vec<Event>` (L48). Avanza un tick completo: orden de turnos, leyes de fin de tick, rotación de rugidos, avance de contador y drena eventos (L49-L101).

**`Simulation::clone`**: NO EXISTE. `struct Simulation` sin `#[derive(Clone)]` ni `impl Clone` (sim/mod.rs:18-20).

### Puerto de eventos y enum de eventos

**`WorldOutput` trait** — `crates/instinct-core/src/event/output.rs:10`
```rust
pub trait WorldOutput {
    fn emit(&mut self, event: &Event); // L13
}
```
Única superficie de eventos del núcleo. Único implementador hoy: `CliOutput` (`crates/instinct-cli/src/output.rs:7-11`).

**`Event` enum** — `crates/instinct-core/src/event/events.rs:13`
```rust
pub enum Event {
    Said { tick: u64, species: String, pos: Pos, text: String },          // L17-L22
    Died { tick: u64, species: String, pos: Pos, cause: String },         // L24-L29
    RuntimeError { tick: u64, species: String, pos: Pos, message: String, line: usize }, // L31-L37
    Damaged { tick: u64, species: String, pos: Pos, target: Pos, amount: i64 }, // L41-L47
    Roared { tick: u64, species: String, pos: Pos, text: String },        // L52-L57
}
```

**`Roar` struct** — `crates/instinct-core/src/world/roars.rs:13`
```rust
pub struct Roar { pub tick: u64, pub creature_id: CreatureId, pub species: String, pub pos: Pos, pub text: String } // L13-L19
```

**`Pos` struct** — `crates/instinct-core/src/event/events.rs:6`
```rust
pub struct Pos { pub x: usize, pub y: usize } // L6-L9
```

## 3. Campos públicos de `World` (fragiles para consumidor externo)

`crates/instinct-core/src/world/world.rs:19-64` son `pub`:
`map_name`, `columns`, `rows`, `terrain`, `terrains`, `objects`, `content`,
`species`, `creatures`, `free_ids`, `ticks`, `events`, `diagnostics`,
`catalog`, `rng` (`RefCell<Box<dyn RngSource>>`, L54), `roars_current`,
`roars_previous`, `output` (`Option<Box<dyn WorldOutput>>`, L64).
Nota: `rng` e `output` usan interior mutabilidad/trait object; expuestos como `pub` por conveniencia del intérprete/sim, no como contrato GUI.

## 4. Clonado

**`Clone for World`** — `crates/instinct-core/src/world/clone_debug.rs:7`
Manual. Reinicia RNG a seed 0 (L24) y `output: None` (L28).
Cita: `rng: RefCell::new(Box::new(Rng::new(0))), // cloning resets the RNG to seed 0` (L24).

**`Clone for Simulation`**: NO EXISTE (sim/mod.rs:18-20). `WorldSnapshot` SÍ deriva `Clone` (snapshot_view.rs:13).

## 5. Huecos para la UI (no existen hoy)

| Hueco | Evidencia |
|-------|-----------|
| Reset / reinicio de simulación | Ningún método `reset`/`restart` en `World` ni `Simulation` |
| Velocidad / control de tiempo | No existe velocidad; `tick` avanza de 1 en 1 (`step_tick`, sim/mod.rs:48) |
| Getter de catálogos de mapas/especies | `resources::load_resources` vive en CLI (`cli/src/run.rs:16`), no en núcleo |
| Clonado que preserve RNG/output | `World::clone` reseta RNG a 0 y quita `output` (clone_debug.rs:24,28) |

## 6. Estabilidad de API para la GUI

- Estable: `snapshot()` (solo lectura, DTO derivado), `WorldOutput::emit`, `Event`, `Roar`, `Pos`, `step_tick`, `set_rng_seed`, `place_creature`/`remove_creature` (devuelven `Result`, no abortan).
- Frágil (pueden romper GUI): campos `pub` de `World` (world.rs:19-64) mutables directamente; `World::clone` reseta RNG (clone_debug.rs:24); `output` es `Box<dyn WorldOutput>` sin `Clone`.
- Ningún `println!` en núcleo (0 matches en `crates/instinct-core/src`) y 0 dependencias externas (`crates/instinct-core/Cargo.toml:74` `[dependencies]` vacío): el núcleo es seguro para incrustar en una GUI sin E/S accidental.