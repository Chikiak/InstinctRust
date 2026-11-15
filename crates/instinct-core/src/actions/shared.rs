//! Shared helpers for actions: direction validation, striking an adjacent
//! cell, and healing the attacker.
//! Thin re-export of the cohesive submodules.
//! The `pub(super) use` items without a local reference are used by
//! the action modules (`attack`, `consume`, etc.) via
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
///
/// It is `pub` (and re-exported as `crate::actions::index_from_i64` in
/// `mod.rs`) because the integration test `tests/actions/index.rs` calls
/// `instinct_core::actions::index_from_i64` directly; keeping it `pub`
/// avoids an `index_from_i64` copy in every action module.
pub fn index_from_i64(n: i64) -> Option<usize> {
    if n < 0 {
        return None;
    }
    usize::try_from(n).ok()
}
