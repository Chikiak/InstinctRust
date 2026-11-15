//! Shared helpers for actions: direction validation, striking an adjacent
//! cell, and healing the attacker.

use super::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::value::Value;
use crate::world::{CellContent, CreatureId, World};

/// Builds an `Event::Damaged` for a strike on cell `(tx, ty)` by the attacker
/// at `pos` belonging to `species`, at `tick`, dealing `amount` damage.
pub(super) fn damaged_event(
    tick: u64,
    species: &str,
    pos: Pos,
    target: Pos,
    amount: i64,
) -> Event {
    Event::Damaged {
        tick,
        species: species.to_string(),
        pos,
        target,
        amount,
    }
}

/// Strikes a creature at `target_id`: reduces its health, kills it at <= 0,
/// and returns damage/died events.
pub(super) fn strike_creature(
    world: &mut World,
    _attacker_id: CreatureId,
    target_id: CreatureId,
    attacker_pos: Pos,
    attacker_species: &str,
    tick: u64,
    tx: usize,
    ty: usize,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
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
    let mut events = vec![damaged_event(
        tick,
        attacker_species,
        attacker_pos,
        Pos { x: tx, y: ty },
        damage,
    )];
    if new_health <= 0 {
        // `Died` identifies the creature that dies (victim), not the
        // attacker; consistent with end_of_tick_creatures and
        // die_by_error.
        let victim_species = creature.species.name.clone();
        let died_pos = Pos { x: creature.x, y: creature.y };
        world.remove_creature(target_id)?;
        events.push(Event::Died {
            tick,
            species: victim_species,
            pos: died_pos,
            cause: "health".to_string(),
        });
    }
    Ok(events)
}

/// Strikes an object at cell `(tx, ty)`: reduces its reserve and returns a
/// damage event.
pub(super) fn strike_object(
    world: &mut World,
    attacker_species: &str,
    attacker_pos: Pos,
    tick: u64,
    tx: usize,
    ty: usize,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
    world.damage_cell_content(tx, ty, damage)?;
    Ok(vec![damaged_event(
        tick,
        attacker_species,
        attacker_pos,
        Pos { x: tx, y: ty },
        damage,
    )])
}

/// Strikes the terrain at cell `(tx, ty)`: reduces its reserve and returns a
/// damage event.
pub(super) fn strike_terrain(
    world: &mut World,
    attacker_species: &str,
    attacker_pos: Pos,
    tick: u64,
    tx: usize,
    ty: usize,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
    let t_idx = world.terrain[ty][tx];
    let t = &mut world.terrains[t_idx];
    t.reserve = t.reserve.checked_sub(damage).ok_or_else(|| {
        Diagnostic::new(
            "attack",
            1,
            format!("overflow damaging terrain at ({tx},{ty})"),
        )
    })?;
    Ok(vec![damaged_event(
        tick,
        attacker_species,
        attacker_pos,
        Pos { x: tx, y: ty },
        damage,
    )])
}

/// Shared logic for "striking an adjacent cell": reduces a creature's health
/// (immediate death at <= 0), an object's reserve, or the terrain reserve of
/// cell `(x+dx, y+dy)` of the attacking creature. Returns damage events. Out
/// of map: no-op, no event, no error. `(0, 0)` targets the terrain under the
/// creature itself.
pub(super) fn strike_cell(
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

    // Destination out of map: no-op, no error, no event.
    if tx < 0 || ty < 0 || tx >= world.columns as i64 || ty >= world.rows as i64 {
        return Ok(Vec::new());
    }
    let tx = tx as usize;
    let ty = ty as usize;

    let mut target = world.content[ty][tx].clone();
    // The attacker's own cell (dx=0, dy=0) strikes the terrain under it,
    // never the creature itself.
    if tx == pos.x && ty == pos.y {
        target = CellContent::Empty;
    }

    match target {
        CellContent::Creature(target_id) => strike_creature(
            world,
            creature_id,
            target_id,
            pos,
            &species,
            tick,
            tx,
            ty,
            damage,
        ),
        CellContent::Object(_obj_idx) => {
            strike_object(world, &species, pos, tick, tx, ty, damage)
        }
        CellContent::Empty => strike_terrain(world, &species, pos, tick, tx, ty, damage),
    }
}

/// Applies `heal` to the attacking creature's health (no cap; can accumulate).
/// Returns a runtime error on overflow.
pub(super) fn heal_attacker(
    world: &mut World,
    creature_id: CreatureId,
    heal: i64,
) -> Result<(), Diagnostic> {
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

/// Parses an integer argument, returning an error naming the argument if it is text.
pub(super) fn parse_int_arg(
    value: &Value,
    action: &str,
    arg_name: &str,
) -> Result<i64, Diagnostic> {
    match value {
        Value::Int(n) => Ok(*n),
        Value::Text(_) => Err(Diagnostic::new(
            action,
            1,
            format!("argument '{arg_name}' must be an integer, got text"),
        )),
    }
}

/// Parses a direction argument: must be an integer in `{-1, 0, 1}`. Names the
/// argument in errors.
pub(super) fn parse_dir_arg(
    value: &Value,
    action: &str,
    arg_name: &str,
) -> Result<i64, Diagnostic> {
    let n = parse_int_arg(value, action, arg_name)?;
    if !((-1..=1).contains(&n)) {
        return Err(Diagnostic::new(
            action,
            1,
            format!(
                "argument '{arg_name}' must be -1, 0, or 1, got {n}"
            ),
        ));
    }
    Ok(n)
}

/// Returns `true` if `(x, y)` is outside the world grid.
pub(super) fn is_out_of_bounds(world: &World, x: i64, y: i64) -> bool {
    x < 0 || y < 0 || x >= world.columns as i64 || y >= world.rows as i64
}

/// No-op result for actions whose target is outside the map.
pub(super) fn out_of_map_noop() -> ActionResult {
    ActionResult {
        sleep_turns: 0,
        events: Vec::new(),
    }
}

/// Converts a `Value` to its textual representation (integers are shown as
/// their decimal string).
pub(super) fn value_to_text(value: &Value) -> String {
    match value {
        Value::Int(n) => n.to_string(),
        Value::Text(s) => s.clone(),
    }
}

/// Returns the creature's position, species name, and the current tick.
pub(super) fn creature_context(
    world: &World,
    creature_id: CreatureId,
) -> Result<(Pos, String, u64), Diagnostic> {
    let pos = world.creature_position(creature_id)?;
    let species = world.creature_species(creature_id)?;
    let tick = world.tick();
    Ok((pos, species, tick))
}
