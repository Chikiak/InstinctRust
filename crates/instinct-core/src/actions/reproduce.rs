//! `reproduce(dx, dy, health)`: creates an offspring of the same species in
//! the adjacent cell (x+dx, y+dy) of the mother if inside the map and it is
//! only terrain (`CellContent::Empty`) and the mother has health strictly
//! greater than `health`. The mother loses `health`; the offspring is born
//! with health = `health`, age 0, no sleep turns, pc at start (`start:`)
//! and empty variables. Its first turn is the next tick (the `step_tick`
//! snapshot excludes it from the current tick). Any failure is a natural
//! abort: it does nothing, but consumes the turn.

use crate::actions::shared::{
    checked_sub_health, creature_context, is_out_of_bounds, noop_result, parse_dir_arg,
    parse_int_arg, unknown_creature_diagnostic, ACTION_REPRODUCE, ARG_DX, ARG_DY, ARG_HEALTH,
};
use crate::actions::ActionContext;
use crate::actions::ActionResult;
use crate::diagnostic::Diagnostic;
use crate::event::Pos;
use crate::species::Species;
use crate::value::Value;
use crate::world::CreatureId;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reproduce;

/// Parses `(dx, dy, health)` from the args, naming the offending argument.
fn parse_input(args: &[Value]) -> Result<(i64, i64, i64), Diagnostic> {
    let dx = parse_dir_arg(&args[0], ACTION_REPRODUCE, ARG_DX)?;
    let dy = parse_dir_arg(&args[1], ACTION_REPRODUCE, ARG_DY)?;
    let health = parse_int_arg(&args[2], ACTION_REPRODUCE, ARG_HEALTH)?;
    Ok((dx, dy, health))
}

/// Mother exists and `mother.health > health`. Returns `(species,
/// mother_health)` or `None` for a natural abort.
fn validate_parents(
    world: &dyn ActionContext,
    creature_id: CreatureId,
    health: i64,
) -> Result<Option<(Rc<Species>, i64)>, Diagnostic> {
    let mother = match world.creature(creature_id) {
        Some(mother) => mother,
        None => return Err(unknown_creature_diagnostic(&creature_id)),
    };
    if mother.health <= health {
        return Ok(None);
    }
    Ok(Some((mother.species.clone(), mother.health)))
}

/// Checked subtraction of the child `health` from the mother's health.
fn mother_health(current: i64, health: i64, pos: Pos) -> Result<i64, Diagnostic> {
    checked_sub_health(current, health, ACTION_REPRODUCE, pos)
}

/// Target cell `(tx, ty)`; `None` when out of map (natural abort).
fn bounds_check(world: &dyn ActionContext, pos: Pos, dx: i64, dy: i64) -> Option<(usize, usize)> {
    let tx = pos.x as i64 + dx;
    let ty = pos.y as i64 + dy;
    if is_out_of_bounds(world, tx, ty) {
        return None;
    }
    Some((tx as usize, ty as usize))
}

/// Places the child with `health` in the free cell `(tx, ty)`. `None` when
/// the cell is occupied (natural abort, mother untouched).
fn place_child(
    world: &mut dyn ActionContext,
    species: &Rc<Species>,
    tx: usize,
    ty: usize,
    health: i64,
) -> Result<Option<CreatureId>, Diagnostic> {
    let child_id = match world.place_creature(species, tx, ty) {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };
    world.set_creature_health(child_id, health)?;
    Ok(Some(child_id))
}

/// Applies `new_mother_health` to the mother.
fn damage_mother(
    world: &mut dyn ActionContext,
    creature_id: CreatureId,
    new_mother_health: i64,
) -> Result<(), Diagnostic> {
    let mother = world
        .creature_mut(creature_id)
        .ok_or_else(|| unknown_creature_diagnostic(&creature_id))?;
    mother.health = new_mother_health;
    Ok(())
}

impl crate::actions::Action for Reproduce {
    fn name(&self) -> &str {
        ACTION_REPRODUCE
    }
    fn arity(&self) -> usize {
        3
    }
    fn consumes_turn(&self) -> bool {
        true
    }
    fn apply(
        &self,
        world: &mut dyn ActionContext,
        creature_id: CreatureId,
        args: &[Value],
    ) -> Result<ActionResult, Diagnostic> {
        // --- parsing ---
        let (dx, dy, health) = parse_input(args)?;

        // --- creature context ---
        let (pos, _species, _tick) = creature_context(world, creature_id)?;

        // --- validation: mother exists and health > `health` ---
        let Some((species, mother_start_health)) = validate_parents(world, creature_id, health)?
        else {
            return Ok(noop_result());
        };

        // --- mother health: checked subtraction ---
        let new_mother_health = mother_health(mother_start_health, health, pos)?;

        // --- bounds check ---
        let Some((tx, ty)) = bounds_check(world, pos, dx, dy) else {
            return Ok(noop_result());
        };

        // --- place child ---
        let Some(_child_id) = place_child(world, &species, tx, ty, health)? else {
            return Ok(noop_result());
        };

        // --- apply damage to mother ---
        damage_mother(world, creature_id, new_mother_health)?;

        Ok(noop_result())
    }
}
