//! Shared helpers for actions: direction validation, striking an adjacent
//! cell, and healing the attacker.
//! Thin re-export of the cohesive submodules.
//! The `pub(super) use` items without a local reference are used by
//! the action modules (`attack`, `consume`, etc.) via
//! `crate::actions::shared::{...}`.

#[allow(unused_imports)]
pub(super) use crate::actions::combat::{
    apply_strike, heal_attacker, strike_cell, strike_creature, strike_object, strike_terrain,
};
#[allow(unused_imports)]
pub(super) use crate::actions::consts::{
    ACTION_ATTACK, ACTION_CONSUME, ACTION_MOVE, ACTION_REPRODUCE, ACTION_ROAR, ACTION_SAY,
    ACTION_WAIT, ARG_DAMAGE, ARG_DX, ARG_DY, ARG_HEAL, ARG_HEALTH, ARG_N, ARG_SPEED, DIAG_CATALOG,
    DIAG_LINE, DIAG_WORLD,
};
#[allow(unused_imports)]
pub(super) use crate::actions::context_helpers::{
    action_result, checked_sub_health, creature_context, is_out_of_bounds, noop_result,
    unknown_creature_diagnostic, value_to_text,
};
#[allow(unused_imports)]
pub(super) use crate::actions::events::damaged_event;
#[allow(unused_imports)]
pub(super) use crate::actions::parse::{parse_dir_arg, parse_dir_args, parse_int_arg};

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
