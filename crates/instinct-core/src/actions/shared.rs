//! Shared helpers for actions: direction validation, striking an adjacent
//! cell, and healing the attacker.
//! Ahora es un re-exportador delgado de los submódulos cohesionados.
//! Los `pub(super) use` que aparecen sin referencia local son usados por
//! los módulos de acción (`attack`, `consume`, etc.) a través de
//! `crate::actions::shared::{...}`.

#[allow(unused_imports)]
pub(super) use crate::actions::consts::{
    ACTION_WAIT, ACTION_MOVE, ACTION_SAY, ACTION_ATTACK, ACTION_CONSUME, ACTION_REPRODUCE, ACTION_ROAR,
    ARG_DX, ARG_DY, ARG_DAMAGE, ARG_HEAL, ARG_SPEED, ARG_HEALTH, ARG_N,
    DIAG_CATALOG, DIAG_WORLD,
};
#[allow(unused_imports)]
pub(super) use crate::actions::parse::{parse_int_arg, parse_dir_arg, parse_dir_args};
#[allow(unused_imports)]
pub(super) use crate::actions::events::damaged_event;
#[allow(unused_imports)]
pub(super) use crate::actions::combat::{
    strike_creature, strike_object, strike_terrain, strike_cell, heal_attacker, apply_strike,
};
#[allow(unused_imports)]
pub(super) use crate::actions::context_helpers::{
    checked_sub_health, is_out_of_bounds, noop_result, value_to_text, creature_context, action_result,
};

/// Converts an integer into a grid index (`usize`). Rejects negatives so they
/// never wrap into a huge index, and rejects values that do not fit in
/// `usize`. Avoids any silent sign conversion when computing the next cell.
pub fn index_from_i64(n: i64) -> Option<usize> {
    if n < 0 {
        return None;
    }
    usize::try_from(n).ok()
}
