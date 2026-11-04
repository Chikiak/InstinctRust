//! Actions: `wait`, `move`, `say`.
//!
//! Actions are stateless types; the catalog keeps them as shared references
//! of a single thread (`Rc<dyn Action>`). The interpreter resolves each name
//! against the catalog validating arity; there are no `if nombre == ...`
//! chains by name.

use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::perception::{Age, Health, Lifespan, Perception, PerceptionCatalog, Random, Tick, X, Y};
use crate::value::Value;
use crate::world::{CellContent, CreatureId, World};
use std::rc::Rc;

/// Result of applying an action: the number of sleep turns (0 = none) and
/// any events to append to the world's event log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResult {
    pub sleep_turns: i64,
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
}

impl Catalog {
    pub fn new() -> Self {
        Catalog {
            actions: Vec::new(),
            perceptions: PerceptionCatalog::new(),
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
}

impl Default for Catalog {
    fn default() -> Self {
        let mut c = Catalog::new();
        let _ = c.register(Rc::new(Wait));
        let _ = c.register(Rc::new(Move));
        let _ = c.register(Rc::new(Say));
        let _ = c.register(Rc::new(Attack));
        let _ = c.register_perception(Box::new(Health));
        let _ = c.register_perception(Box::new(Age));
        let _ = c.register_perception(Box::new(Lifespan));
        let _ = c.register_perception(Box::new(X));
        let _ = c.register_perception(Box::new(Y));
        let _ = c.register_perception(Box::new(Tick));
        let _ = c.register_perception(Box::new(Random));
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
        let sleep_turns = if n < 1 { 0 } else { n - 1 };
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
            if !world.cell_is_free(nx as usize, ny as usize) {
                break;
            }
            x = nx;
            y = ny;
        }

        let _ = world.move_creature_to(creature_id, x as usize, y as usize);
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

        if !((-1..=1).contains(&dx)) {
            return Err(Diagnostic::new(
                "attack",
                1,
                format!("argument 'dx' must be -1, 0, or 1, got {dx}"),
            ));
        }
        if !((-1..=1).contains(&dy)) {
            return Err(Diagnostic::new(
                "attack",
                1,
                format!("argument 'dy' must be -1, 0, or 1, got {dy}"),
            ));
        }

        let pos = world.creature_position(creature_id)?;
        let species = world.creature_species(creature_id)?;
        let tick = world.tick();

        let tx = pos.x as i64 + dx;
        let ty = pos.y as i64 + dy;

        // Destino fuera del mapa: no-op, turno consumido, sin error.
        if tx < 0 || ty < 0 || tx >= world.columns as i64 || ty >= world.rows as i64 {
            return Ok(ActionResult {
                sleep_turns: 0,
                events: Vec::new(),
            });
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
                let creature = world
                    .creatures
                    .iter_mut()
                    .find(|c| c.id == target_id)
                    .ok_or_else(|| {
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
                    // Muerte inmediata: libera la casilla y emite Died en este tick.
                    let died_pos = Pos {
                        x: creature.x,
                        y: creature.y,
                    };
                    world.remove_creature(target_id)?;
                    events.push(Event::Died {
                        tick,
                        species: species.clone(),
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

        Ok(ActionResult {
            sleep_turns: 0,
            events,
        })
    }
}
