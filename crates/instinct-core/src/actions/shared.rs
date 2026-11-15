//! Shared helpers for actions: direction validation, striking an adjacent
//! cell, and healing the attacker.

use super::{ActionContext, ActionResult};
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::value::Value;
use crate::world::{CellContent, CreatureId};

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
    ctx: &mut dyn ActionContext,
    _attacker_id: CreatureId,
    target_id: CreatureId,
    attacker_pos: Pos,
    attacker_species: &str,
    tick: u64,
    tx: usize,
    ty: usize,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
    let creature = ctx.creature_mut(target_id).ok_or_else(|| {
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
        ctx.remove_creature(target_id)?;
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
    ctx: &mut dyn ActionContext,
    attacker_species: &str,
    attacker_pos: Pos,
    tick: u64,
    tx: usize,
    ty: usize,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
    ctx.damage_cell_content(tx, ty, damage)?;
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
    ctx: &mut dyn ActionContext,
    attacker_species: &str,
    attacker_pos: Pos,
    tick: u64,
    tx: usize,
    ty: usize,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
    let t_idx = ctx.terrain_at(tx, ty);
    ctx.damage_terrain_reserve(t_idx, damage)?;
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
    ctx: &mut dyn ActionContext,
    creature_id: CreatureId,
    dx: i64,
    dy: i64,
    damage: i64,
) -> Result<Vec<Event>, Diagnostic> {
    let pos = ctx.creature_position(creature_id)?;
    let species = ctx.creature_species(creature_id)?;
    let tick = ctx.tick();

    let tx = pos.x as i64 + dx;
    let ty = pos.y as i64 + dy;

    // Destination out of map: no-op, no error, no event.
    if tx < 0 || ty < 0 || tx >= ctx.columns() as i64 || ty >= ctx.rows() as i64 {
        return Ok(Vec::new());
    }
    let tx = tx as usize;
    let ty = ty as usize;

    let mut target = ctx.content_at(tx, ty);
    // The attacker's own cell (dx=0, dy=0) strikes the terrain under it,
    // never the creature itself.
    if tx == pos.x && ty == pos.y {
        target = CellContent::Empty;
    }

    match target {
        CellContent::Creature(target_id) => strike_creature(
            ctx,
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
            strike_object(ctx, &species, pos, tick, tx, ty, damage)
        }
        CellContent::Empty => strike_terrain(ctx, &species, pos, tick, tx, ty, damage),
    }
}

/// Applies `heal` to the attacking creature's health (no cap; can accumulate).
/// Returns a runtime error on overflow.
pub(super) fn heal_attacker(
    ctx: &mut dyn ActionContext,
    creature_id: CreatureId,
    heal: i64,
) -> Result<(), Diagnostic> {
    let creature = ctx.creature_mut(creature_id).ok_or_else(|| {
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
pub(super) fn is_out_of_bounds(ctx: &dyn ActionContext, x: i64, y: i64) -> bool {
    x < 0 || y < 0 || x >= ctx.columns() as i64 || y >= ctx.rows() as i64
}

/// No-op result for actions whose target is outside the map.
pub(super) fn out_of_map_noop() -> ActionResult {
    ActionResult {
        sleep_turns: 0,
        events: Vec::new(),
    }
}

/// No-op result for a natural abort (e.g. insufficient health to reproduce):
/// does nothing, but consumes the turn.
pub(super) fn natural_abort_noop() -> ActionResult {
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
    ctx: &dyn ActionContext,
    creature_id: CreatureId,
) -> Result<(Pos, String, u64), Diagnostic> {
    let pos = ctx.creature_position(creature_id)?;
    let species = ctx.creature_species(creature_id)?;
    let tick = ctx.tick();
    Ok((pos, species, tick))
}

/// Converts an integer into a grid index (`usize`). Rejects negatives so they
/// never wrap into a huge index, and rejects values that do not fit in
/// `usize`. Avoids any silent sign conversion when computing the next cell.
pub fn index_from_i64(n: i64) -> Option<usize> {
    if n < 0 {
        return None;
    }
    usize::try_from(n).ok()
}

/// Parses the `dx` and `dy` direction arguments, naming each one in errors.
pub(super) fn parse_dir_args(
    args: &[Value],
    action: &str,
) -> Result<(i64, i64), Diagnostic> {
    let dx = parse_dir_arg(&args[0], action, "dx")?;
    let dy = parse_dir_arg(&args[1], action, "dy")?;
    Ok((dx, dy))
}

/// Builds an `ActionResult` from `sleep_turns` and `events`.
pub(super) fn action_result(sleep_turns: u32, events: Vec<Event>) -> ActionResult {
    ActionResult {
        sleep_turns,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{Event, Pos};
    use crate::value::Value;

    #[test]
    fn parse_dir_args_ok() {
        let args = [Value::Int(1), Value::Int(0)];
        assert_eq!(parse_dir_args(&args, "attack"), Ok((1, 0)));
    }

    #[test]
    fn parse_dir_args_bad_dx() {
        let args = [Value::Int(2), Value::Int(0)];
        let err = parse_dir_args(&args, "consume").unwrap_err();
        assert!(err.message.contains("dx"));
    }

    #[test]
    fn parse_dir_args_bad_dy() {
        let args = [Value::Int(0), Value::Int(-2)];
        let err = parse_dir_args(&args, "move").unwrap_err();
        assert!(err.message.contains("dy"));
    }

    #[test]
    fn parse_dir_args_text_dx() {
        let args = [Value::Text("x".to_string()), Value::Int(0)];
        let err = parse_dir_args(&args, "attack").unwrap_err();
        assert!(err.message.contains("dx"));
    }

    #[test]
    fn parse_dir_args_text_dy() {
        let args = [Value::Int(0), Value::Text("y".to_string())];
        let err = parse_dir_args(&args, "consume").unwrap_err();
        assert!(err.message.contains("dy"));
    }

    #[test]
    fn action_result_constructs() {
        let result = action_result(3, vec![]);
        assert_eq!(
            result,
            ActionResult {
                sleep_turns: 3,
                events: vec![]
            }
        );
    }

    #[test]
    fn action_result_with_events() {
        let result = action_result(
            0,
            vec![Event::Said {
                tick: 5,
                species: "wolf".to_string(),
                pos: Pos { x: 1, y: 2 },
                text: "hi".to_string(),
            }],
        );
        assert_eq!(result.sleep_turns, 0);
        assert_eq!(result.events.len(), 1);
    }
}
