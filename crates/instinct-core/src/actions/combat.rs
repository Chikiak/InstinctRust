//! Strike logic for creature/object/terrain and attacker heal.
use super::context_helpers::{action_result, is_out_of_bounds, unknown_creature_diagnostic};
use super::{consts::*, events::damaged_event};
use crate::actions::{ActionContext, ActionResult, WriteContext};
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::world::{CellContent, CreatureId};

/// Grouped parameters for a strike on a creature. Avoids a signature with
/// many arguments and groups the target and attacker related data.
pub(super) struct StrikeTarget {
    pub target_id: CreatureId,
    pub attacker_pos: Pos,
    pub attacker_species: String,
    pub tick: u64,
    pub tx: usize,
    pub ty: usize,
    pub damage: i64,
}

pub(super) fn strike_creature(
    ctx: &mut dyn WriteContext,
    target: &StrikeTarget,
) -> Result<Vec<Event>, Diagnostic> {
    let creature = ctx
        .creature_mut(target.target_id)
        .ok_or_else(|| unknown_creature_diagnostic(&target.target_id))?;
    let new_health = creature.health.checked_sub(target.damage).ok_or_else(|| {
        Diagnostic::new(
            ACTION_ATTACK,
            DIAG_LINE,
            format!(
                "overflow damaging creature health at ({},{})",
                target.tx, target.ty
            ),
        )
    })?;
    creature.health = new_health;
    let mut events = vec![damaged_event(
        target.tick,
        &target.attacker_species,
        target.attacker_pos,
        Pos {
            x: target.tx,
            y: target.ty,
        },
        target.damage,
    )];
    if new_health <= 0 {
        // `Died` identifies the creature that dies (victim), not the
        // attacker; consistent with end_of_tick_creatures and
        // die_by_error.
        let victim_species = creature.species.name.clone();
        // The position of the `Died` event is the attacked cell (tx, ty),
        // not the victim's internal position, to keep it consistent with the
        // cell the attacker struck.
        let died_pos = Pos {
            x: target.tx,
            y: target.ty,
        };
        ctx.remove_creature(target.target_id)?;
        events.push(Event::Died {
            tick: target.tick,
            species: victim_species,
            pos: died_pos,
            cause: ARG_HEALTH.to_string(),
        });
    }
    Ok(events)
}

pub(super) fn strike_object(
    ctx: &mut dyn WriteContext,
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
    if is_out_of_bounds(ctx, tx, ty) {
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
            &StrikeTarget {
                target_id,
                attacker_pos: pos,
                attacker_species: species,
                tick,
                tx,
                ty,
                damage,
            },
        ),
        CellContent::Object(_obj_idx) => strike_object(ctx, &species, pos, tick, tx, ty, damage),
        CellContent::Empty => strike_terrain(ctx, &species, pos, tick, tx, ty, damage),
    }
}

/// Applies `heal` to the attacking creature's health (no cap; can accumulate).
/// Returns a runtime error on overflow.
pub(super) fn heal_attacker(
    ctx: &mut dyn WriteContext,
    creature_id: CreatureId,
    heal: i64,
) -> Result<(), Diagnostic> {
    let creature = ctx
        .creature_mut(creature_id)
        .ok_or_else(|| unknown_creature_diagnostic(&creature_id))?;
    let new_health = creature.health.checked_add(heal).ok_or_else(|| {
        Diagnostic::new(
            ACTION_CONSUME,
            DIAG_LINE,
            format!(
                "overflow healing creature at ({},{})",
                creature.x, creature.y
            ),
        )
    })?;
    creature.health = new_health;
    Ok(())
}

/// Shared apply path for `attack` and `consume`: strikes an adjacent cell via
/// `strike_cell`, optionally heals the attacker (only when there are events,
/// i.e. not out-of-map), and wraps everything in `action_result(0, events)`.
pub(super) fn apply_strike(
    ctx: &mut dyn ActionContext,
    creature_id: CreatureId,
    dx: i64,
    dy: i64,
    damage: i64,
    heal: Option<i64>,
) -> Result<ActionResult, Diagnostic> {
    let events = strike_cell(ctx, creature_id, dx, dy, damage)?;
    if let Some(h) = heal {
        if !events.is_empty() {
            heal_attacker(ctx, creature_id, h)?;
        }
    }
    Ok(action_result(0, events))
}
