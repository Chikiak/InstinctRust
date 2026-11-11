//! World model: terrain grid, objects, creatures, ticks, and events.
//!
//! The world owns the seeded generator (with interior mutability confined to
//! the generator) and the shared catalog of actions and perceptions. Each
//! creature keeps its own variable map.

use crate::actions::{ActionResult, Catalog};
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos, WorldOutput};
use crate::expr::EvalContext;
use crate::perception::{CreatureView, Nearby, Nearest};
use crate::resources::{Map, Object, Terrain};
use crate::rng::{Rng, RngSource};
use crate::species::Species;
use crate::value::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Stable identifier of a creature. The world owns creatures by id, never by
/// reference; actions receive an id, not a borrow of the creature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CreatureId(pub usize);

/// An element occupying a cell on the terrain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CellContent {
    /// Terrain only.
    Empty,
    /// There is an object on the terrain.
    Object(usize), // index in the world's object catalog
    /// There is a creature on the terrain.
    Creature(CreatureId),
}

/// Un rugido emitido por una criatura en un tick, asociado a su especie,
/// posición y texto. Se guarda con su tick para que, en el siguiente tick,
/// F2 pueda leer los rugidos del tick anterior con `roars_near`/`last_roar`
/// dentro del radio de visión. `creature_id` identifica al emisor para que
/// una criatura no cuente ni lea su propio rugido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roar {
    pub tick: u64,
    pub creature_id: CreatureId,
    pub species: String,
    pub pos: Pos,
    pub text: String,
}

/// State of a creature's turn cycle (pattern State).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureState {
    /// The creature is awake and executes its program normally.
    Awake,
    /// The creature is asleep for `n` more ticks; it does not execute.
    Sleeping(u32),
}

/// A living creature placed on the map.
///
/// `species` es una referencia compartida (`Rc<Species>`) al catálogo del
/// mundo (patrón Flyweight): dos criaturas de la misma especie apuntan a la
/// misma instancia y clonar la criatura clona solo el `Rc`, nunca el `Species`.
#[derive(Debug, Clone)]
pub struct Creature {
    pub id: CreatureId,
    pub species: Rc<Species>,
    pub x: usize,
    pub y: usize,
    /// Current health.
    pub health: i64,
    /// Current age.
    pub age: i64,
    /// Maximum lifespan.
    pub lifespan: i64,
    /// Turn cycle state: Awake or Sleeping(n). Replaces the external
    /// `sleep_turns` counter; the state is managed internally.
    pub state: CreatureState,
    /// Next line index to execute at the start of the next turn.
    pub pc: usize,
    /// Variables owned by this creature, separate from other creatures.
    pub variables: HashMap<String, Value>,
}

/// Memento del estado mutable de una criatura al inicio de un turno.
/// Captura una copia del estado para poder restaurar o diagnosticar tras
/// un turno fallido sin dejar el mundo en un estado intermedio corrupto.
#[derive(Debug, Clone)]
pub struct CreatureMemento {
    pub id: CreatureId,
    pub health: i64,
    pub age: i64,
    pub x: usize,
    pub y: usize,
    pub pc: usize,
    pub state: CreatureState,
    pub variables: HashMap<String, Value>,
}

impl Creature {
    /// Captura el estado mutable actual de esta criatura en un memento.
    pub fn memento(&self) -> CreatureMemento {
        CreatureMemento {
            id: self.id,
            health: self.health,
            age: self.age,
            x: self.x,
            y: self.y,
            pc: self.pc,
            state: self.state.clone(),
            variables: self.variables.clone(),
        }
    }
}

/// The world: terrain grid, objects, creatures, tick counter, and event log.
///
/// `output` (puerto de salida) se excluye de `Clone` y se omite de `Debug`
/// porque `Box<dyn WorldOutput>` no implementa esos traits.
pub struct World {
    pub map_name: String,
    pub columns: usize,
    pub rows: usize,
    /// Terrain of each cell, indexed as [row][column].
    pub terrain: Vec<Vec<usize>>, // index in the terrain catalog
    /// Terrain catalog (reserves are mutated here when a cell's terrain is hit).
    pub terrains: Vec<Terrain>,
    /// Object catalog (reserves are mutated here when a cell's object is hit).
    pub objects: Vec<Object>,
    /// Optional content (object or creature) of each cell.
    pub content: Vec<Vec<CellContent>>,
    /// Catálogo de especies debido por el mundo (Flyweight). Cada entrada es un
    /// `Rc<Species>` compartido; las criaturas guardan clones baratos del mismo
    /// `Rc` sin clonar nunca el `Species` completo.
    pub species: Vec<Rc<Species>>,
    /// Creatures by id: `creatures[id.0]` is `Some(creature)` while the
    /// creature lives and `None` is a free slot ready for reuse. The index is
    /// the stable `CreatureId`, so `creature(id)`/`creature_mut(id)` are O(1)
    /// with no linear search (ECS ligero, sección 4.1).
    pub creatures: Vec<Option<Creature>>,
    /// Free-list of creature ids whose slots can be reused. `place_creature`
    /// pops one here before growing `creatures`; when empty the next id is
    /// `creatures.len()`. Replaces the old `next_creature_id` counter; ids are
    /// stable within a simulation and never invalidated for other creatures.
    free_ids: Vec<usize>,
    /// Number of completed ticks.
    pub ticks: u64,
    /// Events emitted since the last consumer drained them.
    pub events: Vec<Event>,
    pub diagnostics: Vec<Diagnostic>,
    /// Shared catalog of actions and perceptions.
    pub catalog: Catalog,
    /// Seeded pseudo-random generator wrapped as a `RngSource` strategy object.
    /// Interior mutability is confined to this field so `random` can advance it
    /// through an immutable world reference.
    pub rng: RefCell<Box<dyn RngSource>>,
    /// Rugidos emitidos en el tick actual, asociados a su tick y posición.
    /// Al final de cada tick (`sim.rs`) se rotan a `roars_previous`.
    pub roars_current: Vec<Roar>,
    /// Rugidos del tick anterior; legibles durante el tick actual (prepara
    /// `roars_near`/`last_roar` de F2).
    pub roars_previous: Vec<Roar>,
    /// Puerto de salida opcional. Los eventos emitidos por `emit_event` se
    /// reenvían aquí además de guardarse en `self.events`. `None` por defecto:
    /// el mundo funciona igual que antes sin consumidor de eventos.
    pub output: Option<Box<dyn WorldOutput>>,
}

impl World {
    /// Builds a world from a loaded map and the catalogs.
    /// Characters that do not match any terrain or object produce a diagnostic;
    /// the terrain below a cell with an object is the first loaded terrain.
    pub fn from_map(map: &Map, terrains: &[Terrain], objects: &[Object]) -> Self {
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        if terrains.is_empty() {
            diagnostics.push(Diagnostic::new(
                map.name.as_str(),
                1,
                "no terrains loaded to draw the map".to_string(),
            ));
            return World {
                map_name: map.name.clone(),
                columns: 0,
                rows: 0,
                terrain: Vec::new(),
                terrains: terrains.to_vec(),
                objects: objects.to_vec(),
                content: Vec::new(),
                species: Vec::new(),
                creatures: Vec::new(),
                free_ids: Vec::new(),
                ticks: 0,
                events: Vec::new(),
                diagnostics,
                catalog: Catalog::default(),
                rng: RefCell::new(Box::new(Rng::new(0))),
                roars_current: Vec::new(),
                roars_previous: Vec::new(),
                output: None,
            };
        }

        let first_terrain = 0usize;
        let columns = map.rows[0].len();
        let rows = map.rows.len();

        let mut terrain: Vec<Vec<usize>> = Vec::with_capacity(rows);
        let mut content: Vec<Vec<CellContent>> = Vec::with_capacity(rows);

        for (row_idx, row) in map.rows.iter().enumerate() {
            let mut row_terrain: Vec<usize> = Vec::with_capacity(columns);
            let mut row_content: Vec<CellContent> = Vec::with_capacity(columns);
            for c in row.iter() {
                // Look for the character among objects first, then terrains.
                let object_idx = objects.iter().position(|o| o.character == *c);
                if let Some(idx) = object_idx {
                    row_terrain.push(first_terrain);
                    row_content.push(CellContent::Object(idx));
                    continue;
                }
                let terrain_idx = terrains.iter().position(|t| t.character == *c);
                match terrain_idx {
                    Some(idx) => {
                        row_terrain.push(idx);
                        row_content.push(CellContent::Empty);
                    }
                    None => {
                        diagnostics.push(Diagnostic::new(
                            map.name.as_str(),
                            row_idx + 2,
                            format!("unknown character in the map: '{c}'"),
                        ));
                        row_terrain.push(first_terrain);
                        row_content.push(CellContent::Empty);
                    }
                }
            }
            terrain.push(row_terrain);
            content.push(row_content);
        }

        World {
            map_name: map.name.clone(),
            columns,
            rows,
            terrain,
            terrains: terrains.to_vec(),
            objects: objects.to_vec(),
            content,
            species: Vec::new(),
            creatures: Vec::new(),
            free_ids: Vec::new(),
            ticks: 0,
            events: Vec::new(),
            diagnostics,
            catalog: Catalog::default(),
            rng: RefCell::new(Box::new(Rng::new(0))),
            roars_current: Vec::new(),
            roars_previous: Vec::new(),
            output: None,
        }
    }

    // -----------------------------------------------------------------------
    // Creatures: placement, removal, accessors.
    // -----------------------------------------------------------------------

    /// Places a creature on a free cell. Returns the assigned `CreatureId` or
    /// a `Diagnostic` naming the coordinates when the cell is out of bounds or
    /// already occupied. The world is unchanged on failure.
    ///
    /// `species` es una referencia al catálogo compartido (Flyweight): la
    /// criatura guarda `Rc::clone(species)`, nunca un `Species` clonado, y el
    /// mundo asegura tener su propio `Rc` en `species` (el catálogo dueño).
    pub fn place_creature(
        &mut self,
        species: &Rc<Species>,
        x: usize,
        y: usize,
    ) -> Result<CreatureId, Diagnostic> {
        if x >= self.columns || y >= self.rows {
            return Err(Diagnostic::new(
                "world",
                0,
                format!(
                    "cannot place creature at ({x},{y}): outside the map ({}x{})",
                    self.columns, self.rows
                ),
            ));
        }
        if !self.cell_is_free(x, y) {
            return Err(Diagnostic::new(
                "world",
                0,
                format!("cannot place creature at ({x},{y}): cell is occupied",),
            ));
        }

        // El mundo es dueño de su copia del `Rc<Species>` en el catálogo;
        // si ya está registrado (mismo puntero) se reutiliza la entrada.
        if !self.species.iter().any(|owned| Rc::ptr_eq(owned, species)) {
            self.species.push(Rc::clone(species));
        }

        // Reusa un hueco libre si hay uno; si no, crece el almacén. El índice
        // es el id estable de la nueva criatura.
        let id = match self.free_ids.pop() {
            Some(slot) => slot,
            None => {
                self.creatures.push(None);
                self.creatures.len() - 1
            }
        };
        let health = species.health;
        let lifespan = species.lifespan;
        let creature = Creature {
            id: CreatureId(id),
            species: Rc::clone(species), // flyweight: clona el Rc, no el Species
            x,
            y,
            health,
            age: 0,
            lifespan,
            state: CreatureState::Awake,
            pc: 0,
            variables: HashMap::new(),
        };
        self.content[y][x] = CellContent::Creature(CreatureId(id));
        self.creatures[id] = Some(creature);
        Ok(CreatureId(id))
    }

    /// Removes the creature with the given id, freeing its cell. Returns a
    /// diagnostic if the id does not exist.
    pub fn remove_creature(&mut self, creature_id: CreatureId) -> Result<(), Diagnostic> {
        let pos = self.creature_position(creature_id)?; // O(1): índice directo
        self.content[pos.y][pos.x] = CellContent::Empty;
        // Libera el hueco sin invalidar los ids de las otras criaturas.
        self.creatures[creature_id.0] = None;
        self.free_ids.push(creature_id.0);
        Ok(())
    }

    /// Ids de las criaturas vivas, en orden ascendente de id (orden del
    /// almacén). `sim.step_tick` itera sobre estos ids; ninguna busca lineal.
    pub fn live_creature_ids(&self) -> Vec<CreatureId> {
        self.creatures
            .iter()
            .enumerate()
            .filter_map(|(index, slot)| slot.as_ref().map(|_| CreatureId(index)))
            .collect()
    }

    /// Returns the position of the creature, or a diagnostic if unknown.
    /// O(1): indexa directamente en `creatures[id.0]`.
    pub fn creature_position(&self, creature_id: CreatureId) -> Result<Pos, Diagnostic> {
        self.creature(creature_id)
            .map(|c| Pos { x: c.x, y: c.y })
            .ok_or_else(|| {
                Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
            })
    }

    /// Returns the species name of the creature, or a diagnostic if unknown.
    pub fn creature_species(&self, creature_id: CreatureId) -> Result<String, Diagnostic> {
        self.creature(creature_id)
            .map(|c| c.species.name.clone())
            .ok_or_else(|| {
                Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
            })
    }

    /// Returns the creature with the given id, or `None`. O(1): indexación
    /// directa; `None` tanto si el id está fuera de rango como si su hueco
    /// está libre.
    pub(crate) fn creature(&self, id: CreatureId) -> Option<&Creature> {
        self.creatures.get(id.0).and_then(Option::as_ref)
    }

    /// Returns the creature with the given id mutably, or `None`. O(1).
    pub(crate) fn creature_mut(&mut self, id: CreatureId) -> Option<&mut Creature> {
        self.creatures.get_mut(id.0).and_then(Option::as_mut)
    }

    /// Returns the value of a creature variable, or `None` if unassigned or
    /// unknown creature.
    pub fn variable(&self, creature_id: CreatureId, name: &str) -> Option<Value> {
        self.creature(creature_id)
            .and_then(|c| c.variables.get(name).cloned())
    }

    /// Fija la salud de la criatura con el id dado. Devuelve un diagnóstico
    /// si el id no existe; nunca indexa fuera de rango. `reproduce` lo usa
    /// para ajustar la salud de la cría tras `place_creature` (que usa
    /// `species.health`) al valor indicado en la acción.
    pub fn set_creature_health(
        &mut self,
        creature_id: CreatureId,
        health: i64,
    ) -> Result<(), Diagnostic> {
        let creature = self.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        creature.health = health;
        Ok(())
    }

    /// Assigns a variable on a creature. Rejects any name the catalog knows
    /// as an action, a perception, a function, or a constant.
    pub fn set_variable(
        &mut self,
        creature_id: CreatureId,
        name: &str,
        value: Value,
    ) -> Result<(), Diagnostic> {
        if self.catalog.is_action_name(name)
            || self.catalog.is_perception_name(name)
            || self.catalog.is_function_name(name)
            || self.catalog.is_constant_name(name)
        {
            return Err(Diagnostic::new(
                "world",
                1,
                format!("cannot assign to '{name}': it is a reserved name"),
            ));
        }
        let creature = self.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 1, format!("unknown creature id {creature_id:?}"))
        })?;
        creature.variables.insert(name.to_string(), value);
        Ok(())
    }

    /// Restaura el estado de una criatura desde un memento capturado al
    /// inicio del turno. Devuelve `false` si la criatura ya no existe
    /// (sin hacer nada). Se usa para deshacer cambios parciales cuando un
    /// turno falla con error de ejecución.
    pub fn restore_creature_from_memento(&mut self, memento: &CreatureMemento) -> bool {
        if let Some(creature) = self.creature_mut(memento.id) {
            creature.health = memento.health;
            creature.age = memento.age;
            creature.x = memento.x;
            creature.y = memento.y;
            creature.pc = memento.pc;
            creature.state = memento.state.clone();
            creature.variables = memento.variables.clone();
            true
        } else {
            false
        }
    }

    /// Seeds the world's generator. Two worlds with the same seed produce the
    /// same `random` sequence and turn order.
    pub fn set_rng_seed(&mut self, seed: u64) {
        *self.rng.borrow_mut() = Box::new(Rng::new(seed));
    }

    /// Injects a `RngSource` strategy (e.g. a deterministic `TestRng` for
    /// tests). Replaces the world's current generator.
    pub fn set_rng_source(&mut self, rng: Box<dyn RngSource>) {
        *self.rng.borrow_mut() = rng;
    }

    /// Conecta un puerto de salida al mundo. Los eventos emitidos por
    /// `emit_event` se reenvían al puerto además de guardarse en `self.events`.
    /// El puerto es opcional: si no se conecta nada, el mundo funciona como
    /// antes. Nunca se propaga un error del puerto al tick.
    pub fn set_output(&mut self, output: Box<dyn WorldOutput>) {
        self.output = Some(output);
    }

    /// Returns the current tick count.
    pub fn tick(&self) -> u64 {
        self.ticks
    }

    /// Advances the tick counter by one.
    pub fn advance_tick(&mut self) {
        self.ticks += 1;
    }

    /// Appends an event to the world's event log and forwards it to the
    /// output port if one is connected. A failure in the port never aborts
    /// the tick.
    pub fn emit_event(&mut self, event: Event) {
        // Reenvía al puerto antes de almacenar: el puerto toma prestada una
        // referencia al evento, luego el evento se mueve al log.
        if let Some(output) = self.output.as_mut() {
            output.emit(&event);
        }
        self.events.push(event);
    }

    /// Drains all pending events, returning them and leaving the log empty.
    pub fn drain_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Registra un rugido emitido por la criatura `creature_id` desde `pos`
    /// con el texto `text` en el tick actual. `roar` no mata ni daña: solo
    /// deja constancia del mensaje para que F2 lo lea el tick siguiente con
    /// `roars_near`/`last_roar`.
    pub fn add_roar(&mut self, creature_id: CreatureId, species: String, pos: Pos, text: String) {
        self.roars_current.push(Roar {
            tick: self.ticks,
            creature_id,
            species,
            pos,
            text,
        });
    }

    /// Devuelve los rugidos del tick anterior (disponibles durante el tick
    /// actual). Vacío hasta que `sim.rs` ejecute la primera rotación.
    pub fn previous_roars(&self) -> &[Roar] {
        &self.roars_previous
    }

    /// Nombre del terreno de la casilla `(x, y)`, o `None` si está fuera del
    /// mapa o su índice no está cubierto por el catálogo de terrenos.
    pub fn terrain_name_at(&self, x: usize, y: usize) -> Option<&str> {
        self.terrain
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|&t| self.terrains.get(t))
            .map(|t| t.name.as_str())
    }

    /// Nombre del objeto en la casilla `(x, y)`, o `None` si no hay objeto
    /// o está fuera del mapa.
    pub fn object_name_at(&self, x: usize, y: usize) -> Option<&str> {
        self.content
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|c| match c {
                CellContent::Object(idx) => self.objects.get(*idx).map(|o| o.name.as_str()),
                _ => None,
            })
    }

    /// Identificador de la criatura en la casilla `(x, y)`, o `None` si no
    /// hay criatura o está fuera del mapa.
    pub fn creature_at(&self, x: usize, y: usize) -> Option<CreatureId> {
        self.content
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|c| match c {
                CellContent::Creature(id) => Some(*id),
                _ => None,
            })
    }

    /// Reserva actual del terreno de la casilla `(x, y)`, leída del catálogo
    /// de terrenos del mundo (no de la casilla), o `None` si está fuera del
    /// mapa. Dos casillas con el mismo terreno comparten la misma reserva.
    pub fn reserve_at(&self, x: usize, y: usize) -> Option<i64> {
        self.terrain
            .get(y)
            .and_then(|row| row.get(x))
            .and_then(|&t| self.terrains.get(t))
            .map(|t| t.reserve)
    }

    /// Número de criaturas vivas de la facción dada, excluyendo a `exclude`.
    /// Todas las criaturas del mundo están vivas (las muertas se retiran al
    /// morir).
    pub fn count_creatures_by_faction_excluding(
        &self,
        faction: &str,
        exclude: CreatureId,
    ) -> usize {
        self.creatures
            .iter()
            .flatten()
            .filter(|c| c.id != exclude && c.species.faction == faction)
            .count()
    }

    /// Calcula la estructura `Nearby` para la criatura dada con un solo
    /// recorrido determinista del mundo: criaturas por id ascendente, rejilla
    /// por filas y columnas ascendentes, rugidos del tick anterior en orden
    /// de vector. Distancia de Chebyshev; radio <= vision; casilla propia
    /// excluida de conteos y de "más cercanos".
    pub fn nearby(&self, creature_id: CreatureId) -> Nearby {
        let self_pos = match self.creature_position(creature_id) {
            Ok(p) => p,
            Err(_) => return Nearby::empty(),
        };
        let self_creature = match self.creature(creature_id) {
            Some(c) => c,
            None => return Nearby::empty(),
        };
        let self_faction = self_creature.species.faction.as_str();
        let vision = self_creature.species.vision;

        let mut allies = 0i64;
        let mut enemies = 0i64;
        let mut objects = 0i64;
        let mut nearest_ally: Option<Nearest> = None;
        let mut nearest_enemy: Option<Nearest> = None;
        let mut nearest_object: Option<Nearest> = None;

        // Pass 1: criaturas vivas por id ascendente (excluida la propia).
        for c in self.creatures.iter().flatten() {
            if c.id == creature_id {
                continue;
            }
            let dx = c.x as i64 - self_pos.x as i64;
            let dy = c.y as i64 - self_pos.y as i64;
            let dist = dx.abs().max(dy.abs());
            if dist > vision {
                continue;
            }
            if c.species.faction == self_faction {
                allies += 1;
                if nearest_ally.is_none() || dist < nearest_ally.unwrap().dist {
                    nearest_ally = Some(Nearest {
                        dist,
                        dx: dx.signum(),
                        dy: dy.signum(),
                    });
                }
            } else {
                enemies += 1;
                if nearest_enemy.is_none() || dist < nearest_enemy.unwrap().dist {
                    nearest_enemy = Some(Nearest {
                        dist,
                        dx: dx.signum(),
                        dy: dy.signum(),
                    });
                }
            }
        }

        // Pass 2: rejilla por filas ascendentes, columnas ascendentes, para objetos.
        for y in 0..self.rows {
            for x in 0..self.columns {
                if let CellContent::Object(_) = &self.content[y][x] {
                    let dx = x as i64 - self_pos.x as i64;
                    let dy = y as i64 - self_pos.y as i64;
                    let dist = dx.abs().max(dy.abs());
                    if dist > vision {
                        continue;
                    }
                    objects += 1;
                    if nearest_object.is_none() || dist < nearest_object.unwrap().dist {
                        nearest_object = Some(Nearest {
                            dist,
                            dx: dx.signum(),
                            dy: dy.signum(),
                        });
                    }
                }
            }
        }

        // Pass 3: rugidos del tick anterior en orden del vector.
        let mut roars = 0i64;
        let mut last_roar = String::new();
        for roar in self.roars_previous.iter() {
            if roar.creature_id == creature_id {
                continue;
            }
            let dx = roar.pos.x as i64 - self_pos.x as i64;
            let dy = roar.pos.y as i64 - self_pos.y as i64;
            let dist = dx.abs().max(dy.abs());
            if dist > vision {
                continue;
            }
            roars += 1;
            last_roar = roar.text.clone();
        }

        Nearby {
            allies,
            enemies,
            objects,
            ally: nearest_ally,
            enemy: nearest_enemy,
            object: nearest_object,
            roars,
            last_roar,
        }
    }

    /// Marca los rugidos del tick actual como "del tick anterior" y limpia los
    /// actuales. Llama `sim.rs` al final de cada tick para que un rugido no
    /// persista más allá del tick siguiente al de su emisión.
    pub fn rotate_roars(&mut self) {
        self.roars_previous = std::mem::take(&mut self.roars_current);
    }

    /// Returns true if the cell at (x, y) is inside the map and has no object
    /// and no creature (terrain only).
    pub fn cell_is_free(&self, x: usize, y: usize) -> bool {
        if x >= self.columns || y >= self.rows {
            return false;
        }
        matches!(self.content[y][x], CellContent::Empty)
    }

    /// Moves a creature to a new cell. The caller must have verified the cell
    /// is free and in bounds. Updates both the creature and the grid.
    pub fn move_creature_to(
        &mut self,
        creature_id: CreatureId,
        x: usize,
        y: usize,
    ) -> Result<(), Diagnostic> {
        if x >= self.columns || y >= self.rows {
            return Err(Diagnostic::new(
                "world",
                0,
                format!(
                    "cannot move creature {creature_id:?} to ({x},{y}): outside the map ({}x{})",
                    self.columns, self.rows
                ),
            ));
        }
        let old = self.creature_position(creature_id)?;
        self.content[old.y][old.x] = CellContent::Empty;
        if let Some(creature) = self.creature_mut(creature_id) {
            creature.x = x;
            creature.y = y;
        }
        self.content[y][x] = CellContent::Creature(creature_id);
        Ok(())
    }

    /// Daña (o cura, si `damage` es negativo) el contenido de la celda (x, y):
    /// reduce la reserva del objeto o del terreno de esa celda con aritmética
    /// checked. Fuera del mapa es un no-op sin error. Las criaturas no se
    /// tocan aquí (su salud la gestiona el llamante). Un objeto cuyo reserve
    /// llega a 0 o menos es retirado por la ley de fin de tick existente.
    pub fn damage_cell_content(
        &mut self,
        x: usize,
        y: usize,
        damage: i64,
    ) -> Result<(), Diagnostic> {
        if x >= self.columns || y >= self.rows {
            return Ok(()); // fuera del mapa: no-op, sin error
        }
        match &self.content[y][x] {
            CellContent::Object(obj_idx) => {
                let obj = &mut self.objects[*obj_idx];
                obj.reserve = obj.reserve.checked_sub(damage).ok_or_else(|| {
                    Diagnostic::new("world", 0, format!("overflow damaging object at ({x},{y})"))
                })?;
            }
            CellContent::Empty => {
                let t_idx = self.terrain[y][x];
                let t = &mut self.terrains[t_idx];
                t.reserve = t.reserve.checked_sub(damage).ok_or_else(|| {
                    Diagnostic::new(
                        "world",
                        0,
                        format!("overflow damaging terrain at ({x},{y})"),
                    )
                })?;
            }
            CellContent::Creature(_) => {
                // La salud de criatura la gestiona el llamante (Attack).
            }
        }
        Ok(())
    }

    /// Applies the result of an action: sets the creature's turn state and
    /// appends events. `sleep_turns > 0` means `Sleeping(sleep_turns)`;
    /// zero means `Awake`.
    pub fn apply_action_result(
        &mut self,
        creature_id: CreatureId,
        result: ActionResult,
    ) -> Result<(), Diagnostic> {
        if let Some(creature) = self.creature_mut(creature_id) {
            creature.state = if result.sleep_turns > 0 {
                CreatureState::Sleeping(result.sleep_turns)
            } else {
                CreatureState::Awake
            };
        } else {
            return Err(Diagnostic::new(
                "world",
                0,
                format!("unknown creature id {creature_id:?}"),
            ));
        }
        for event in result.events {
            self.emit_event(event);
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // End of tick laws.
    // -----------------------------------------------------------------------

    /// Applies the end-of-tick laws to every creature: -1 health, +1 age, and
    /// death when health <= 0 or age >= lifespan. Returns `Died` events for
    /// creatures that die and removes them from the map.
    ///
    /// Recorre el almacén por índice (ids vivos), O(1) por acceso, sin
    /// búsqueda lineal ni `dead_ids.contains` en bucles.
    ///
    /// La salud y la edad se calculan con aritmética comprobada. Si
    /// `health - 1` o `age + 1` desborda (p. ej. vida en `i64::MIN` o edad en
    /// `i64::MAX`) se trata como un error de ejecución: la criatura muere con
    /// causa `"error"` sin provocar pánico en debug ni en release.
    pub fn end_of_tick_creatures(&mut self) -> Vec<Event> {
        let mut died: Vec<Event> = Vec::new();
        let mut dead_indices: Vec<usize> = Vec::new();
        // (index, next_health, next_age) de las criaturas que sobreviven, para
        // aplicar el cambio tras el escaneo sin indexar dos veces.
        let mut survivors: Vec<(usize, i64, i64)> = Vec::new();

        for (index, slot) in self.creatures.iter_mut().enumerate() {
            let Some(creature) = slot else { continue }; // hueco libre
            match (creature.health.checked_sub(1), creature.age.checked_add(1)) {
                (Some(next_health), Some(next_age)) => {
                    if next_health <= 0 || next_age >= creature.lifespan {
                        dead_indices.push(index);
                        let cause = if next_health <= 0 { "vida" } else { "edad" };
                        died.push(Event::Died {
                            tick: self.ticks,
                            species: creature.species.name.clone(),
                            pos: Pos {
                                x: creature.x,
                                y: creature.y,
                            },
                            cause: cause.to_string(),
                        });
                    } else {
                        survivors.push((index, next_health, next_age));
                    }
                }
                _ => {
                    // Desbordamiento: error de ejecución, la criatura muere.
                    dead_indices.push(index);
                    died.push(Event::Died {
                        tick: self.ticks,
                        species: creature.species.name.clone(),
                        pos: Pos {
                            x: creature.x,
                            y: creature.y,
                        },
                        cause: "error".to_string(),
                    });
                }
            }
        }

        // Aplica salud y edad a las sobrevivientes.
        for (index, next_health, next_age) in survivors {
            if let Some(creature) = self.creatures[index].as_mut() {
                creature.health = next_health;
                creature.age = next_age;
            }
        }
        // Elimina las muertas (por vida, edad o error de ejecución), liberando
        // sus huecos sin invalidar otros ids.
        for index in dead_indices {
            let _ = self.remove_creature(CreatureId(index));
        }
        died
    }

    /// Regenerates terrain reserves up to `resource_max` and removes objects
    /// whose reserve is 0 or less, freeing their cells.
    ///
    /// Opera sobre los catálogos propios del mundo (dueño único): es la misma
    /// reserva que `damage_cell_content` muta, no una copia, de modo que daño
    /// y regeneración nunca trabajan sobre reservas distintas.
    pub fn end_of_tick_terrain_and_objects(&mut self) -> Vec<Pos> {
        let mut freed: Vec<Pos> = Vec::new();

        // Regenerate each terrain once per tick (not once per cell).
        for terrain in self.terrains.iter_mut() {
            if terrain.reserve < terrain.resource_max {
                let new_reserve = terrain.reserve + terrain.regen;
                terrain.reserve = new_reserve.min(terrain.resource_max);
            }
        }

        // Remove exhausted objects and free their cells.
        for y in 0..self.rows {
            for x in 0..self.columns {
                if let CellContent::Object(obj_idx) = &self.content[y][x] {
                    if self.objects[*obj_idx].reserve <= 0 {
                        self.content[y][x] = CellContent::Empty;
                        freed.push(Pos { x, y });
                    }
                }
            }
        }
        freed
    }

    // -----------------------------------------------------------------------
    // Drawing.
    // -----------------------------------------------------------------------

    /// Draws the world in ASCII: the creature (first letter of its species)
    /// if there is one, otherwise the object's character if there is one,
    /// otherwise the terrain's character.
    ///
    /// Valida contra los catálogos propios del mundo (dueño único), por lo que
    /// no recibe catálogos como argumentos.
    ///
    /// Returns a `Diagnostic` when the grid references a creature that does
    /// not exist, or when an object/terrain index in the grid is not covered
    /// by the world's catalogs.
    pub fn draw(&self) -> Result<String, Diagnostic> {
        let mut output = String::new();
        for row in 0..self.rows {
            for column in 0..self.columns {
                let c = match &self.content[row][column] {
                    CellContent::Creature(id) => {
                        let creature = self.creature(*id).ok_or_else(|| {
                            Diagnostic::new(
                                "world",
                                0,
                                format!("grid references missing creature {id:?}"),
                            )
                        })?;
                        creature.species.name.chars().next().ok_or_else(|| {
                            Diagnostic::new(
                                "world",
                                0,
                                format!("creature {id:?} has an empty species name"),
                            )
                        })?
                    }
                    CellContent::Object(idx) => {
                        if *idx >= self.objects.len() {
                            return Err(Diagnostic::new(
                                "world",
                                0,
                                format!(
                                    "grid references object index {idx} but the object catalog has {} entries",
                                    self.objects.len()
                                ),
                            ));
                        }
                        self.objects[*idx].character
                    }
                    CellContent::Empty => {
                        let t_idx = self.terrain[row][column];
                        if t_idx >= self.terrains.len() {
                            return Err(Diagnostic::new(
                                "world",
                                0,
                                format!(
                                    "grid references terrain index {t_idx} but the terrain catalog has {} entries",
                                    self.terrains.len()
                                ),
                            ));
                        }
                        self.terrains[t_idx].character
                    }
                };
                output.push(c);
            }
            output.push('\n');
        }
        Ok(output)
    }
}

impl Clone for World {
    fn clone(&self) -> Self {
        World {
            map_name: self.map_name.clone(),
            columns: self.columns,
            rows: self.rows,
            terrain: self.terrain.clone(),
            terrains: self.terrains.clone(),
            objects: self.objects.clone(),
            content: self.content.clone(),
            species: self.species.clone(),
            creatures: self.creatures.clone(),
            free_ids: self.free_ids.clone(),
            ticks: self.ticks,
            events: self.events.clone(),
            diagnostics: self.diagnostics.clone(),
            catalog: self.catalog.clone(),
            rng: RefCell::new(Box::new(Rng::new(0))), // clonar resetea el RNG a seed 0
            roars_current: self.roars_current.clone(),
            roars_previous: self.roars_previous.clone(),
            // El puerto de salida no se clona: el clonado empieza sin puerto.
            output: None,
        }
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("map_name", &self.map_name)
            .field("columns", &self.columns)
            .field("rows", &self.rows)
            .field("terrain", &self.terrain)
            .field("terrains", &self.terrains)
            .field("objects", &self.objects)
            .field("content", &self.content)
            .field("species", &self.species)
            .field("creatures", &self.creatures)
            .field("free_ids", &self.free_ids)
            .field("ticks", &self.ticks)
            .field("events", &self.events)
            .field("diagnostics", &self.diagnostics)
            .field("catalog", &self.catalog)
            .field("rng", &self.rng)
            .field("roars_current", &self.roars_current)
            .field("roars_previous", &self.roars_previous)
            .field("output", &self.output.is_some())
            .finish()
    }
}

/// Read-only view of a creature and its world, used to evaluate perceptions.
/// Nothing here mutates the world except `random`, which advances the
/// generator through interior mutability.
#[derive(Debug)]
struct CreatureSnapshot<'a> {
    world: &'a World,
    creature_id: CreatureId,
}

impl CreatureView for CreatureSnapshot<'_> {
    fn health(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.health)
            .unwrap_or(0)
    }

    fn age(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.age)
            .unwrap_or(0)
    }

    fn lifespan(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.lifespan)
            .unwrap_or(0)
    }

    fn x(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.x as i64)
            .unwrap_or(0)
    }

    fn y(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.y as i64)
            .unwrap_or(0)
    }

    fn tick(&self) -> u64 {
        self.world.ticks
    }

    fn random(&self) -> i64 {
        self.world.rng.borrow_mut().next_bounded(99)
    }

    fn vision(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .map(|c| c.species.vision)
            .unwrap_or(0)
    }

    fn species_name(&self) -> &str {
        self.world
            .creature(self.creature_id)
            .map(|c| c.species.name.as_str())
            .unwrap_or("")
    }

    fn faction_name(&self) -> &str {
        self.world
            .creature(self.creature_id)
            .map(|c| c.species.faction.as_str())
            .unwrap_or("")
    }

    fn terrain_here(&self) -> &str {
        self.world
            .creature(self.creature_id)
            .and_then(|c| self.world.creature_position(c.id).ok())
            .and_then(|p| self.world.terrain_name_at(p.x, p.y))
            .unwrap_or("")
    }

    fn reserve_here(&self) -> i64 {
        self.world
            .creature(self.creature_id)
            .and_then(|c| self.world.creature_position(c.id).ok())
            .and_then(|p| self.world.reserve_at(p.x, p.y))
            .unwrap_or(0)
    }

    fn width(&self) -> i64 {
        self.world.columns as i64
    }

    fn height(&self) -> i64 {
        self.world.rows as i64
    }

    fn total_creatures(&self) -> i64 {
        // Solo cuenta criaturas vivas: los huecos libres (`None`) no son
        // criaturas.
        let n = self.world.creatures.iter().flatten().count();
        let present = self.world.creature(self.creature_id).is_some();
        (n.saturating_sub(if present { 1 } else { 0 })) as i64
    }

    fn total_allies(&self) -> i64 {
        let Some(creature) = self.world.creature(self.creature_id) else {
            return 0;
        };
        self.world
            .count_creatures_by_faction_excluding(&creature.species.faction, self.creature_id)
            as i64
    }

    fn total_enemies(&self) -> i64 {
        // Todo lo que no es aliado (y no es la propia criatura) es enemigo.
        self.total_creatures() - self.total_allies()
    }

    fn nearby(&self) -> Nearby {
        self.world.nearby(self.creature_id)
    }

    fn see(&self, x: i64, y: i64) -> i64 {
        // Radius check first: Chebyshev distance from creature's cell.
        let self_pos = match self.world.creature_position(self.creature_id) {
            Ok(p) => p,
            Err(_) => return -1, // NONE: unknown creature
        };
        let self_creature = match self.world.creature(self.creature_id) {
            Some(c) => c,
            None => return -1,
        };
        let dx = x - self_pos.x as i64;
        let dy = y - self_pos.y as i64;
        let dist = dx.abs().max(dy.abs());
        if dist > self_creature.species.vision {
            return -1; // NONE: out of radius
        }

        // Out of map: OBJECT (border is a wall).
        if x < 0 || y < 0 || x >= self.world.columns as i64 || y >= self.world.rows as i64 {
            return 1; // OBJECT
        }
        let x = x as usize;
        let y = y as usize;

        // Own cell contains a creature: ALLY.
        match &self.world.content[y][x] {
            CellContent::Creature(target_id) => {
                let target = match self.world.creature(*target_id) {
                    Some(c) => c,
                    None => return 0,
                };
                if target.species.faction == self_creature.species.faction {
                    2 // ALLY
                } else {
                    3 // ENEMY
                }
            }
            CellContent::Object(_) => 1, // OBJECT
            CellContent::Empty => 0,     // GROUND
        }
    }

    fn name(&self, x: i64, y: i64) -> &str {
        // Out of radius or out of map: empty text.
        let self_pos = match self.world.creature_position(self.creature_id) {
            Ok(p) => p,
            Err(_) => return "",
        };
        let self_creature = match self.world.creature(self.creature_id) {
            Some(c) => c,
            None => return "",
        };
        let dx = x - self_pos.x as i64;
        let dy = y - self_pos.y as i64;
        let dist = dx.abs().max(dy.abs());
        if dist > self_creature.species.vision {
            return "";
        }

        if x < 0 || y < 0 || x >= self.world.columns as i64 || y >= self.world.rows as i64 {
            return "";
        }
        let x = x as usize;
        let y = y as usize;

        match &self.world.content[y][x] {
            CellContent::Creature(target_id) => self
                .world
                .creature(*target_id)
                .map(|c| c.species.name.as_str())
                .unwrap_or(""),
            CellContent::Object(obj_idx) => self.world.objects[*obj_idx].name.as_str(),
            CellContent::Empty => self.world.terrain_name_at(x, y).unwrap_or(""),
        }
    }
}

/// Evaluation context for a single creature. Resolves perceptions first, then
/// variables. Execution reads the world immutably; the world applies actions
/// afterward. Holds a snapshot of the creature for function calls and a handle
/// to the turn-state variables so assignments made during the same turn are
/// visible to subsequent expressions.
pub struct CreatureContext<'a> {
    world: &'a World,
    creature_id: CreatureId,
    snapshot: CreatureSnapshot<'a>,
    variables: Rc<RefCell<HashMap<String, Value>>>,
}

impl<'a> CreatureContext<'a> {
    pub fn new(
        world: &'a World,
        creature_id: CreatureId,
        variables: Rc<RefCell<HashMap<String, Value>>>,
    ) -> Self {
        let snapshot = CreatureSnapshot { world, creature_id };
        CreatureContext {
            world,
            creature_id,
            snapshot,
            variables,
        }
    }
}

impl<'a> EvalContext for CreatureContext<'a> {
    fn resolve(&self, name: &str) -> Option<Value> {
        // Perceptions take priority over variables.
        if let Some(perception) = self.world.catalog.resolve_perception(name) {
            return Some(perception.read(&self.snapshot));
        }
        // Constants resolve next.
        if let Some(value) = self.world.catalog.resolve_constant(name) {
            return Some(Value::Int(value));
        }
        // Turn-state variables (assignments made this turn) shadow persisted
        // variables stored on the creature in the world.
        self.variables
            .borrow()
            .get(name)
            .cloned()
            .or_else(|| self.world.variable(self.creature_id, name))
    }

    fn view(&self) -> Option<&dyn CreatureView> {
        Some(&self.snapshot)
    }
}
