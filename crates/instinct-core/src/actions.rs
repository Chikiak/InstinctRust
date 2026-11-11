//! Actions: `wait`, `move`, `say`, `attack`, `consume`, `reproduce`.
//!
//! Actions are stateless types; the catalog keeps them as shared references
//! of a single thread (`Rc<dyn Action>`). The interpreter resolves each name
//! against the catalog validating arity; there are no `if nombre == ...`
//! chains by name.

use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::functions::{Function, FunctionCatalog};
use crate::perception::{
    Age, AlliesNear, AllyDist, AllyDx, AllyDy, EnemiesNear, EnemyDist, EnemyDx, EnemyDy, Faction,
    Health, Height, LastRoar, Lifespan, ObjectDist, ObjectDx, ObjectDy, ObjectsNear, Perception,
    PerceptionCatalog, Random, ReserveHere, RoarsNear, Species, TerrainHere, Tick, TotalAllies,
    TotalCreatures, TotalEnemies, Vision, Width, X, Y,
};
use crate::value::Value;
use crate::world::{CellContent, CreatureId, World};
use std::rc::Rc;

/// Valida que `dx`/`dy` estén en `{-1, 0, 1}`, devolviendo un error que
/// nombra el argumento cuando no lo están.
fn validate_direction(action: &str, dx: i64, dy: i64) -> Result<(), Diagnostic> {
    if !((-1..=1).contains(&dx)) {
        return Err(Diagnostic::new(
            action,
            1,
            format!("argument 'dx' must be -1, 0, or 1, got {dx}"),
        ));
    }
    if !((-1..=1).contains(&dy)) {
        return Err(Diagnostic::new(
            action,
            1,
            format!("argument 'dy' must be -1, 0, or 1, got {dy}"),
        ));
    }
    Ok(())
}

/// Lógica compartida de "golpear casilla adyacente": reduce la vida de una
/// criatura (muerte inmediata a <= 0), la reserva de un objeto o la reserva
/// del terreno de la casilla `(x+dx, y+dy)` de la criatura atacante.
/// Devuelve los eventos de daño. Fuera del mapa: no-op sin evento ni error.
/// `(0, 0)` apunta al terreno bajo la propia criatura.
fn strike_cell(
    world: &mut World,
    creature_id: CreatureId,
    dx: i64,
    dy: i64,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
    let pos = world.creature_position(creature_id)?;
    let species = world.creature_species(creature_id)?;
    let tick = world.tick();

    let tx = pos.x as i64 + dx;
    let ty = pos.y as i64 + dy;

    // Destino fuera del mapa: no-op, sin error, sin evento.
    if tx < 0 || ty < 0 || tx >= world.columns as i64 || ty >= world.rows as i64 {
        return Ok(Vec::new());
    }
    let tx = tx as usize;
    let ty = ty as usize;

    let mut target = world.content[ty][tx].clone();
    // La propia casilla del atacante (dx=0,dy=0) golpea el terreno bajo ella,
    // nunca a la criatura misma.
    if tx == pos.x && ty == pos.y {
        target = CellContent::Empty;
    }
    let mut events: Vec<Event> = Vec::new();

    match target {
        CellContent::Creature(target_id) => {
            let creature = world.creature_mut(target_id).ok_or_else(|| {
                Diagnostic::new("world", 0, format!("unknown creature id {target_id:?}"))
            })?;
            let new_health = creature.health.checked_sub(damage).ok_or_else(|| {
                Diagnostic::new(
                    "attack",
                    1,
                    format!("overflow damaging creature health at ({tx},{ty})"),
                )
            })?;
            creature.health = new_health;
            events.push(Event::Damaged {
                tick,
                species: species.clone(),
                pos,
                target: Pos { x: tx, y: ty },
                amount: damage,
            });
            if new_health <= 0 {
                // `Died` identifica a la criatura que muere (víctima), no al
                // atacante; consistente con end_of_tick_creatures y die_by_error.
                let victim_species = creature.species.name.clone();
                let died_pos = Pos {
                    x: creature.x,
                    y: creature.y,
                };
                world.remove_creature(target_id)?;
                events.push(Event::Died {
                    tick,
                    species: victim_species,
                    pos: died_pos,
                    cause: "vida".to_string(),
                });
            }
        }
        CellContent::Object(_obj_idx) => {
            world.damage_cell_content(tx, ty, damage)?;
            events.push(Event::Damaged {
                tick,
                species: species.clone(),
                pos,
                target: Pos { x: tx, y: ty },
                amount: damage,
            });
        }
        CellContent::Empty => {
            let t_idx = world.terrain[ty][tx];
            let t = &mut world.terrains[t_idx];
            t.reserve = t.reserve.checked_sub(damage).ok_or_else(|| {
                Diagnostic::new(
                    "attack",
                    1,
                    format!("overflow damaging terrain at ({tx},{ty})"),
                )
            })?;
            events.push(Event::Damaged {
                tick,
                species: species.clone(),
                pos,
                target: Pos { x: tx, y: ty },
                amount: damage,
            });
        }
    }

    Ok(events)
}

/// Aplica `heal` a la salud de la criatura atacante (sin tope; puede acumularse).
/// Devuelve un error de runtime si hay overflow.
fn heal_attacker(world: &mut World, creature_id: CreatureId, heal: i64) -> Result<(), Diagnostic> {
    let creature = world.creature_mut(creature_id).ok_or_else(|| {
        Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
    })?;
    let new_health = creature.health.checked_add(heal).ok_or_else(|| {
        Diagnostic::new(
            "consume",
            1,
            format!(
                "overflow healing creature at ({},{})",
                creature.x, creature.y
            ),
        )
    })?;
    creature.health = new_health;
    Ok(())
}

/// Result of applying an action: the number of sleep turns (0 = awake) and
/// any events to append to the world's event log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub sleep_turns: u32,
    pub events: Vec<Event>,
}

/// Trait every action type implements. Stateless: no per-creature state here.
pub trait Action: std::fmt::Debug {
    /// Action name as it appears in the language (e.g. `wait`, `move`, `say`).
    fn name(&self) -> &str;
    /// Number of evaluated arguments the action expects.
    fn arity(&self) -> usize;
    /// Whether executing this action consumes a turn. `say` does not.
    fn consumes_turn(&self) -> bool;
    /// Apply the action with already-evaluated arguments on the world for the
    /// given creature id. Returns an `ActionResult`.
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic>;
}

/// Catalog of actions and perceptions by name. Resolves each kind of name
/// against its own registry, validating arity for actions.
#[derive(Debug, Clone)]
pub struct Catalog {
    actions: Vec<Rc<dyn Action>>,
    perceptions: PerceptionCatalog,
    functions: FunctionCatalog,
}

impl Catalog {
    pub fn new() -> Self {
        Catalog {
            actions: Vec::new(),
            perceptions: PerceptionCatalog::new(),
            functions: FunctionCatalog::new(),
        }
    }

    /// Registers an action type. Returns an error if an action with the same
    /// name is already registered.
    pub fn register(&mut self, action: Rc<dyn Action>) -> Result<(), Diagnostic> {
        if self.actions.iter().any(|a| a.name() == action.name()) {
            return Err(Diagnostic::new(
                "catalog",
                1,
                format!("action '{}' already registered", action.name()),
            ));
        }
        self.actions.push(action);
        Ok(())
    }

    /// Registers a perception type. Returns an error if a perception with the
    /// same name is already registered.
    pub fn register_perception(
        &mut self,
        perception: Box<dyn Perception>,
    ) -> Result<(), Diagnostic> {
        self.perceptions.register(perception)
    }

    /// Resolves an action by name, validating arity. Returns a diagnostic
    /// naming the argument if unknown or wrong arity.
    pub fn resolve(&self, name: &str, arg_count: usize) -> Result<Rc<dyn Action>, Diagnostic> {
        match self.actions.iter().find(|a| a.name() == name) {
            Some(action) => {
                if action.arity() != arg_count {
                    return Err(Diagnostic::new(
                        "catalog",
                        1,
                        format!(
                            "action '{}' expects {} arguments, got {}",
                            name,
                            action.arity(),
                            arg_count
                        ),
                    ));
                }
                Ok(Rc::clone(action))
            }
            None => Err(Diagnostic::new(
                "catalog",
                1,
                format!("unknown action '{name}'"),
            )),
        }
    }

    /// Resolves a perception by name, borrowing it from the catalog. Returns
    /// `None` when the name is not a registered perception.
    pub fn resolve_perception(&self, name: &str) -> Option<&dyn Perception> {
        self.perceptions.resolve(name)
    }

    /// Returns true if `name` is a registered action.
    pub fn is_action_name(&self, name: &str) -> bool {
        self.actions.iter().any(|a| a.name() == name)
    }

    /// Returns true if `name` is a registered perception.
    pub fn is_perception_name(&self, name: &str) -> bool {
        self.perceptions.contains(name)
    }

    /// Resolves a function by name, validating arity. Returns a diagnostic
    /// naming the function if unknown or wrong arity.
    pub fn resolve_function(
        &self,
        name: &str,
        arg_count: usize,
    ) -> Result<Rc<dyn Function>, Diagnostic> {
        self.functions.resolve_function(name, arg_count)
    }

    /// Returns true if `name` is a registered function.
    pub fn is_function_name(&self, name: &str) -> bool {
        self.functions.is_function_name(name)
    }

    /// Resolves a constant by name. Returns `None` when the name is not a
    /// registered constant.
    pub fn resolve_constant(&self, name: &str) -> Option<i64> {
        self.functions.resolve_constant(name)
    }

    /// Returns true if `name` is a registered constant.
    pub fn is_constant_name(&self, name: &str) -> bool {
        self.functions.is_constant_name(name)
    }

    /// Returns a reference to the function catalog.
    pub fn functions(&self) -> &FunctionCatalog {
        &self.functions
    }
}

impl Default for Catalog {
    fn default() -> Self {
        let mut c = Catalog::new();
        let _ = c.register(Rc::new(Wait));
        let _ = c.register(Rc::new(Move));
        let _ = c.register(Rc::new(Say));
        let _ = c.register(Rc::new(Attack));
        let _ = c.register(Rc::new(Consume));
        let _ = c.register(Rc::new(Reproduce));
        let _ = c.register(Rc::new(Roar));
        let _ = c.register_perception(Box::new(Health));
        let _ = c.register_perception(Box::new(Age));
        let _ = c.register_perception(Box::new(Lifespan));
        let _ = c.register_perception(Box::new(X));
        let _ = c.register_perception(Box::new(Y));
        let _ = c.register_perception(Box::new(Tick));
        let _ = c.register_perception(Box::new(Random));
        let _ = c.register_perception(Box::new(Vision));
        let _ = c.register_perception(Box::new(Species));
        let _ = c.register_perception(Box::new(Faction));
        let _ = c.register_perception(Box::new(TerrainHere));
        let _ = c.register_perception(Box::new(ReserveHere));
        let _ = c.register_perception(Box::new(Width));
        let _ = c.register_perception(Box::new(Height));
        let _ = c.register_perception(Box::new(TotalCreatures));
        let _ = c.register_perception(Box::new(TotalAllies));
        let _ = c.register_perception(Box::new(TotalEnemies));
        let _ = c.register_perception(Box::new(AlliesNear));
        let _ = c.register_perception(Box::new(EnemiesNear));
        let _ = c.register_perception(Box::new(ObjectsNear));
        let _ = c.register_perception(Box::new(EnemyDist));
        let _ = c.register_perception(Box::new(EnemyDx));
        let _ = c.register_perception(Box::new(EnemyDy));
        let _ = c.register_perception(Box::new(AllyDist));
        let _ = c.register_perception(Box::new(AllyDx));
        let _ = c.register_perception(Box::new(AllyDy));
        let _ = c.register_perception(Box::new(ObjectDist));
        let _ = c.register_perception(Box::new(ObjectDx));
        let _ = c.register_perception(Box::new(ObjectDy));
        let _ = c.register_perception(Box::new(RoarsNear));
        let _ = c.register_perception(Box::new(LastRoar));
        // Register reserved functions and constants.
        let _ = c.functions.register(Rc::new(crate::functions::See));
        let _ = c.functions.register(Rc::new(crate::functions::Name));
        let _ = c.functions.register_constant("NONE", -1);
        let _ = c.functions.register_constant("GROUND", 0);
        let _ = c.functions.register_constant("OBJECT", 1);
        let _ = c.functions.register_constant("ALLY", 2);
        let _ = c.functions.register_constant("ENEMY", 3);
        c
    }
}

// ---------------------------------------------------------------------------
// wait(n): sleep n-1 turns; n < 1 counts as 1 (zero sleep); text fails.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wait;

impl Action for Wait {
    fn name(&self) -> &str {
        "wait"
    }
    fn arity(&self) -> usize {
        1
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        _world: &mut World,
        _creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let n = match &args[0] {
            Value::Int(n) => *n,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "wait",
                    1,
                    "argument 'n' must be an integer, got text".to_string(),
                ));
            }
        };
        let sleep_turns = n.max(1) as u32;
        Ok(ActionResult {
            sleep_turns,
            events: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// move(dx, dy, speed): advances up to `speed` cells one by one, stopping
// before the first cell outside the map, with an object, or with a creature.
// Direction (0, 0) or speed <= 0: does not move and does not fail.
// dx or dy outside {-1, 0, 1}, or text arguments: error naming the argument.
// ---------------------------------------------------------------------------

/// Convierte un entero en un índice de rejilla (`usize`). Rechaza los
/// negativos para que nunca se envelopen a un índice enorme, y rechaza
/// valores que no quepan en `usize`. Es defensivo en `Move::apply` porque
/// `validate_direction` ya limita `dx`/`dy` a `{-1, 0, 1}`, pero evita cualquier
/// conversión silenciosa de signo.
pub fn index_from_i64(n: i64) -> Option<usize> {
    if n < 0 {
        return None;
    }
    usize::try_from(n).ok()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Move;

impl Action for Move {
    fn name(&self) -> &str {
        "move"
    }
    fn arity(&self) -> usize {
        3
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let dx = parse_move_arg(&args[0], "dx")?;
        let dy = parse_move_arg(&args[1], "dy")?;
        let speed = match &args[2] {
            Value::Int(s) => *s,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "move",
                    1,
                    "argument 'speed' must be an integer, got text".to_string(),
                ));
            }
        };

        // Direction (0, 0) or speed <= 0: no move, no error.
        if (dx == 0 && dy == 0) || speed <= 0 {
            return Ok(ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            });
        }

        // dx or dy outside {-1, 0, 1}: error naming the argument.
        if !((-1..=1).contains(&dx)) {
            return Err(Diagnostic::new(
                "move",
                1,
                format!("argument 'dx' must be -1, 0, or 1, got {dx}"),
            ));
        }
        if !((-1..=1).contains(&dy)) {
            return Err(Diagnostic::new(
                "move",
                1,
                format!("argument 'dy' must be -1, 0, or 1, got {dy}"),
            ));
        }

        let pos = world.creature_position(creature_id)?;
        let mut x = pos.x as i64;
        let mut y = pos.y as i64;

        for _ in 0..speed {
            let nx = x + dx;
            let ny = y + dy;
            // Rechaza coordenadas negativas: nunca se convierten a usize.
            let Some(nxi) = index_from_i64(nx) else { break };
            let Some(nyi) = index_from_i64(ny) else { break };
            if !world.cell_is_free(nxi, nyi) {
                break;
            }
            x = nx;
            y = ny;
        }

        // Destino final: coordenadas no negativas (x/y nunca se vuelven
        // negativas por el break anterior, pero se valida por defensa).
        let Some(xi) = index_from_i64(x) else {
            return Err(Diagnostic::new(
                "move",
                1,
                "coordenada de destino fuera de la rejilla".to_string(),
            ));
        };
        let Some(yi) = index_from_i64(y) else {
            return Err(Diagnostic::new(
                "move",
                1,
                "coordenada de destino fuera de la rejilla".to_string(),
            ));
        };
        let _ = world.move_creature_to(creature_id, xi, yi);
        Ok(ActionResult {
            sleep_turns: 0,
            events: Vec::new(),
        })
    }
}

/// Parses a move direction argument, returning it as i64 or an error naming
/// the argument if it is text.
fn parse_move_arg(value: &Value, arg_name: &str) -> Result<i64, Diagnostic> {
    match value {
        Value::Int(n) => Ok(*n),
        Value::Text(_) => Err(Diagnostic::new(
            "move",
            1,
            format!("argument '{arg_name}' must be an integer, got text"),
        )),
    }
}

// ---------------------------------------------------------------------------
// say(val): emits a Said event with the creature's current position and tick,
// does not consume a turn. A number is shown as text.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Say;

impl Action for Say {
    fn name(&self) -> &str {
        "say"
    }
    fn arity(&self) -> usize {
        1
    }
    fn consumes_turn(&self) -> bool {
        false
    }
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let text = match &args[0] {
            Value::Int(n) => n.to_string(),
            Value::Text(s) => s.clone(),
        };
        let pos = world.creature_position(creature_id)?;
        let species = world.creature_species(creature_id)?;
        let tick = world.tick();
        Ok(ActionResult {
            sleep_turns: 0,
            events: vec![Event::Said {
                tick,
                species,
                pos,
                text,
            }],
        })
    }
}

// ---------------------------------------------------------------------------
// attack(dx, dy, damage): golpea la casilla adyacente (x+dx, y+dy) de la
// criatura atacante. Reduce la vida de una criatura, la reserva de un objeto
// o la reserva del terreno de esa casilla. dx/dy fuera de {-1,0,1} o
// argumentos de texto fallan nombrando el argumento. Destino fuera del mapa:
// no-op, turno consumido, sin error. Una criatura que cae a vida <= 0 muere
// en el acto (casilla libre, Died causa vida) en el mismo tick.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attack;

impl Action for Attack {
    fn name(&self) -> &str {
        "attack"
    }
    fn arity(&self) -> usize {
        3
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let dx = parse_move_arg(&args[0], "dx")?;
        let dy = parse_move_arg(&args[1], "dy")?;
        let damage = match &args[2] {
            Value::Int(d) => *d,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "attack",
                    1,
                    "argument 'damage' must be an integer, got text".to_string(),
                ));
            }
        };
        validate_direction("attack", dx, dy)?;

        let events = strike_cell(world, creature_id, dx, dy, damage)?;
        Ok(ActionResult {
            sleep_turns: 0,
            events,
        })
    }
}

// ---------------------------------------------------------------------------
// consume(dx, dy, damage, heal): igual que attack sobre la casilla adyacente
// y, además, la criatura atacante gana `heal` puntos de vida (sin tope).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consume;

impl Action for Consume {
    fn name(&self) -> &str {
        "consume"
    }
    fn arity(&self) -> usize {
        4
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let dx = parse_move_arg(&args[0], "dx")?;
        let dy = parse_move_arg(&args[1], "dy")?;
        let damage = match &args[2] {
            Value::Int(d) => *d,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "consume",
                    1,
                    "argument 'damage' must be an integer, got text".to_string(),
                ));
            }
        };
        let heal = match &args[3] {
            Value::Int(h) => *h,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "consume",
                    1,
                    "argument 'heal' must be an integer, got text".to_string(),
                ));
            }
        };
        validate_direction("consume", dx, dy)?;

        // Fuera del mapa: no-op, ni daño ni curación, turno consumido.
        let pos = world.creature_position(creature_id)?;
        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;
        if tx < 0 || ty < 0 || tx >= world.columns as i64 || ty >= world.rows as i64 {
            return Ok(ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            });
        }

        let events = strike_cell(world, creature_id, dx, dy, damage)?;
        heal_attacker(world, creature_id, heal)?;
        Ok(ActionResult {
            sleep_turns: 0,
            events,
        })
    }
}

// ---------------------------------------------------------------------------
// reproduce(dx, dy, health): crea una cría de la misma especie en la casilla
// adyacente (x+dx, y+dy) de la madre si está dentro del mapa y es solo
// terreno (`CellContent::Empty`) y la madre tiene vida estrictamente mayor
// que `health`. La madre pierde `health`; la cría nace con vida = `health`,
// edad 0, sin turnos de sueño, pc al inicio (`start:`) y variables vacías.
// Su primer turno es el tick siguiente (el snapshot de `step_tick` la excluye
// del tick actual). Cualquier fallo es aborto natural: no hace nada, pero
// consume turno.
// ---------------------------------------------------------------------------

/// Acción sin efecto: aborto natural. El turno se consume porque
/// `consumes_turn()` es true.
fn abort_reproduction() -> ActionResult {
    ActionResult {
        sleep_turns: 0,
        events: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reproduce;

impl Action for Reproduce {
    fn name(&self) -> &str {
        "reproduce"
    }
    fn arity(&self) -> usize {
        3
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let dx = parse_move_arg(&args[0], "dx")?;
        let dy = parse_move_arg(&args[1], "dy")?;
        let health = match &args[2] {
            Value::Int(h) => *h,
            Value::Text(_) => {
                return Err(Diagnostic::new(
                    "reproduce",
                    1,
                    "argument 'health' must be an integer, got text".to_string(),
                ));
            }
        };
        validate_direction("reproduce", dx, dy)?;

        let pos = world.creature_position(creature_id)?;
        let mother = world.creature(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        // La madre debe tener vida estrictamente mayor que `health`.
        if mother.health <= health {
            return Ok(abort_reproduction());
        }
        let species = mother.species.clone();
        let mother_health = mother.health;

        // Nueva salud de la madre con aritmética checked, antes de mutar nada.
        let new_mother_health = mother_health.checked_sub(health).ok_or_else(|| {
            Diagnostic::new(
                "reproduce",
                1,
                format!("overflow reducing mother health at ({},{})", pos.x, pos.y),
            )
        })?;

        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;
        // Fuera del mapa: aborto natural.
        if tx < 0 || ty < 0 || tx >= world.columns as i64 || ty >= world.rows as i64 {
            return Ok(abort_reproduction());
        }
        // `place_creature` valida bounds y celda libre: celda ocupada ->
        // aborto natural sin tocar a la madre.
        let child_id = match world.place_creature(&species, tx as usize, ty as usize) {
            Ok(id) => id,
            Err(_) => return Ok(abort_reproduction()),
        };
        // `place_creature` inicializa con `species.health`; ajustar a `health`.
        world.set_creature_health(child_id, health)?;

        let mother = world.creature_mut(creature_id).ok_or_else(|| {
            Diagnostic::new("world", 0, format!("unknown creature id {creature_id:?}"))
        })?;
        mother.health = new_mother_health;

        Ok(abort_reproduction())
    }
}

// ---------------------------------------------------------------------------
// roar(val): emite un mensaje de rugido que queda disponible el tick
// siguiente. Cualquier valor (entero o texto) es válido; un entero se muestra
// como su representación textual. Consume turno. No daña ni mata.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roar;

impl Action for Roar {
    fn name(&self) -> &str {
        "roar"
    }
    fn arity(&self) -> usize {
        1
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        world: &mut World,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        let text = match &args[0] {
            Value::Int(n) => n.to_string(),
            Value::Text(s) => s.clone(),
        };
        let pos = world.creature_position(creature_id)?;
        let species = world.creature_species(creature_id)?;
        let tick = world.tick();
        world.add_roar(creature_id, species.clone(), pos, text.clone());
        Ok(ActionResult {
            sleep_turns: 0,
            events: vec![Event::Roared {
                tick,
                species,
                pos,
                text,
            }],
        })
    }
}
